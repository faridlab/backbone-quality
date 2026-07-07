//! Golden cases — the inspection-verdict oracle + the NC→CAPA→close flow. A reading is judged against
//! its parameter's criterion; the inspection is accepted iff every reading passes.

mod common;

use backbone_quality::application::service::quality_events::QualityEvent;
use backbone_quality::application::service::quality_write_service::{
    NewInspection, NewNonConformance, NewQualityAction, NewReading, NewTemplate,
    NewTemplateParameter, QualityError, QualityWriteService,
};
use common::*;
use uuid::Uuid;

/// A template: numeric Diameter in [9.5, 10.5] + a non-numeric Color (manual pass/fail).
async fn diameter_color_template(svc: &QualityWriteService, company: Uuid, item: Uuid) -> Uuid {
    svc.create_template(NewTemplate {
        company_id: company, template_name: "Widget QC".into(), item_id: Some(item),
        parameters: vec![
            NewTemplateParameter { parameter_name: "Diameter".into(), numeric: true,
                min_value: Some(dec("9.5")), max_value: Some(dec("10.5")), spec_text: None },
            NewTemplateParameter { parameter_name: "Color".into(), numeric: false,
                min_value: None, max_value: None, spec_text: Some("matte black".into()) },
        ],
    }).await.unwrap()
}

/// QGC-1 — every reading meets its criterion → ACCEPTED, event carries accepted=true.
#[tokio::test]
async fn qgc1_all_in_spec_accepted() {
    let pool = pool().await;
    let svc = QualityWriteService::new(pool.clone());
    let sink = CapturingSink::new();
    let (company, item) = (Uuid::new_v4(), Uuid::new_v4());
    let tpl = diameter_color_template(&svc, company, item).await;

    let out = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 5,
        readings: vec![
            NewReading { parameter_name: "Diameter".into(), value: Some(dec("10.0")), manual_pass: None, remarks: None },
            NewReading { parameter_name: "Color".into(), value: None, manual_pass: Some(true), remarks: None },
        ],
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    assert!(out.accepted);
    let status: String = sqlx::query_scalar("SELECT status::text FROM quality.quality_inspections WHERE id=$1")
        .bind(out.inspection_id).fetch_one(&pool).await.unwrap();
    assert_eq!(status, "accepted");
    let ev = sink.events().into_iter().find_map(|e| match e {
        QualityEvent::QualityInspectionCompleted(c) => Some(c), _ => None }).unwrap();
    assert!(ev.accepted);
}

/// QGC-2 — one reading out of spec → REJECTED (and only the failing reading is marked rejected).
#[tokio::test]
async fn qgc2_out_of_spec_rejected() {
    let pool = pool().await;
    let svc = QualityWriteService::new(pool.clone());
    let sink = CapturingSink::new();
    let (company, item) = (Uuid::new_v4(), Uuid::new_v4());
    let tpl = diameter_color_template(&svc, company, item).await;

    let out = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 5,
        readings: vec![
            NewReading { parameter_name: "Diameter".into(), value: Some(dec("11.2")), manual_pass: None, remarks: None }, // > 10.5
            NewReading { parameter_name: "Color".into(), value: None, manual_pass: Some(true), remarks: None },
        ],
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    assert!(!out.accepted);
    let status: String = sqlx::query_scalar("SELECT status::text FROM quality.quality_inspections WHERE id=$1")
        .bind(out.inspection_id).fetch_one(&pool).await.unwrap();
    assert_eq!(status, "rejected");
    let (diam, color): (String, String) = sqlx::query_as(
        "SELECT (SELECT result::text FROM quality.quality_inspection_readings WHERE inspection_id=$1 AND parameter_name='Diameter'),
                (SELECT result::text FROM quality.quality_inspection_readings WHERE inspection_id=$1 AND parameter_name='Color')")
        .bind(out.inspection_id).fetch_one(&pool).await.unwrap();
    assert_eq!(diam, "rejected", "the out-of-range diameter failed");
    assert_eq!(color, "accepted", "the in-spec color still passed");
}

/// QGC-3 — the CAPA flow: a rejected inspection raises an NC, gathers an action, and closes once the
/// action is completed.
#[tokio::test]
async fn qgc3_nc_capa_close_flow() {
    let pool = pool().await;
    let svc = QualityWriteService::new(pool.clone());
    let sink = CapturingSink::new();
    let (company, item) = (Uuid::new_v4(), Uuid::new_v4());
    let tpl = diameter_color_template(&svc, company, item).await;
    let out = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 1,
        readings: vec![
            NewReading { parameter_name: "Diameter".into(), value: Some(dec("8.0")), manual_pass: None, remarks: None }, // < 9.5
            NewReading { parameter_name: "Color".into(), value: None, manual_pass: Some(true), remarks: None },
        ],
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    assert!(!out.accepted);

    let nc = svc.raise_non_conformance(NewNonConformance {
        company_id: company, subject: "Undersized widgets".into(), source_inspection_id: Some(out.inspection_id),
        item_id: Some(item), severity: "high".into(), description: None,
    }, dt("2026-07-07T09:05:00Z"), &sink).await.unwrap();
    let action = svc.add_quality_action(NewQualityAction {
        non_conformance_id: nc, action_type: "corrective".into(), procedure_id: None,
        description: "Re-calibrate the lathe".into(), due_date: None,
    }).await.unwrap();

    // Can't close while the action is open.
    assert!(matches!(svc.close_non_conformance(nc, dt("2026-07-07T10:00:00Z"), &sink).await, Err(QualityError::InvalidState(_))));
    let st: String = sqlx::query_scalar("SELECT status::text FROM quality.non_conformances WHERE id=$1")
        .bind(nc).fetch_one(&pool).await.unwrap();
    assert_eq!(st, "in_progress", "adding an action advances the NC");

    svc.complete_action(action, dt("2026-07-07T11:00:00Z")).await.unwrap();
    svc.close_non_conformance(nc, dt("2026-07-07T12:00:00Z"), &sink).await.unwrap();
    let st2: String = sqlx::query_scalar("SELECT status::text FROM quality.non_conformances WHERE id=$1")
        .bind(nc).fetch_one(&pool).await.unwrap();
    assert_eq!(st2, "closed");
}

/// QGC-4 — the input guards.
#[tokio::test]
async fn qgc4_validation() {
    let pool = pool().await;
    let svc = QualityWriteService::new(pool.clone());
    let sink = LoggingSink;
    let (company, item) = (Uuid::new_v4(), Uuid::new_v4());

    let no_param = svc.create_template(NewTemplate {
        company_id: company, template_name: "X".into(), item_id: None, parameters: vec![],
    }).await;
    assert!(matches!(no_param, Err(QualityError::Invalid(_))), "template needs a parameter");

    let bad_range = svc.create_template(NewTemplate {
        company_id: company, template_name: "X".into(), item_id: None,
        parameters: vec![NewTemplateParameter { parameter_name: "D".into(), numeric: true,
            min_value: Some(dec("10")), max_value: Some(dec("5")), spec_text: None }],
    }).await;
    assert!(matches!(bad_range, Err(QualityError::Invalid(_))), "numeric min must be <= max");

    let tpl = diameter_color_template(&svc, company, item).await;
    let no_reading = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 1, readings: vec![],
    }, dt("2026-07-07T09:00:00Z"), &sink).await;
    assert!(matches!(no_reading, Err(QualityError::Invalid(_))), "inspection needs a reading");

    let bad_param = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 1,
        readings: vec![NewReading { parameter_name: "Weight".into(), value: Some(dec("1")), manual_pass: None, remarks: None }],
    }, dt("2026-07-07T09:00:00Z"), &sink).await;
    assert!(matches!(bad_param, Err(QualityError::Invalid(_))), "a reading must name a template parameter");
}
