//! Integrity probes — the inspection/CAPA invariants that keep the quality funnel honest.

mod common;

use backbone_quality::application::service::quality_events::QualityEvent;
use backbone_quality::application::service::quality_write_service::{
    NewInspection, NewNonConformance, NewQualityAction, NewReading, NewTemplate,
    NewTemplateParameter, QualityError, QualityWriteService,
};
use common::*;
use uuid::Uuid;

async fn numeric_template(svc: &QualityWriteService, company: Uuid) -> Uuid {
    svc.create_template(NewTemplate {
        company_id: company, template_name: "T".into(), item_id: None,
        parameters: vec![NewTemplateParameter { parameter_name: "Diameter".into(), numeric: true,
            min_value: Some(dec("9.5")), max_value: Some(dec("10.5")), spec_text: None }],
    }).await.unwrap()
}
async fn inspect_diameter(svc: &QualityWriteService, company: Uuid, tpl: Uuid, item: Uuid, v: &str, sink: &LoggingSink) -> (Uuid, bool) {
    let out = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 1,
        readings: vec![NewReading { parameter_name: "Diameter".into(), value: Some(dec(v)), manual_pass: None, remarks: None }],
    }, dt("2026-07-07T09:00:00Z"), sink).await.unwrap();
    (out.inspection_id, out.accepted)
}

/// IP-1 — an NC cannot cite a NON-rejected inspection (you don't raise an NC on a passing inspection).
#[tokio::test]
async fn ip1_nc_requires_rejected_inspection() {
    let pool = pool().await;
    let svc = QualityWriteService::new(pool.clone());
    let sink = LoggingSink;
    let (company, item) = (Uuid::new_v4(), Uuid::new_v4());
    let tpl = numeric_template(&svc, company).await;
    let (accepted_insp, ok) = inspect_diameter(&svc, company, tpl, item, "10.0", &sink).await;
    assert!(ok);
    let _ = &pool;

    let err = svc.raise_non_conformance(NewNonConformance {
        company_id: company, subject: "bogus".into(), source_inspection_id: Some(accepted_insp),
        item_id: Some(item), severity: "low".into(), description: None,
    }, dt("2026-07-07T09:05:00Z"), &sink).await.unwrap_err();
    assert!(matches!(err, QualityError::InvalidState(_)), "can't raise an NC on an accepted inspection");
}

/// IP-2 — an NC closes only when every action is completed; an incomplete action blocks the close.
#[tokio::test]
async fn ip2_close_blocked_by_incomplete_action() {
    let pool = pool().await;
    let svc = QualityWriteService::new(pool.clone());
    let sink = LoggingSink;
    let company = Uuid::new_v4();
    let nc = svc.raise_non_conformance(NewNonConformance {
        company_id: company, subject: "issue".into(), source_inspection_id: None, item_id: None,
        severity: "medium".into(), description: None,
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    let a1 = svc.add_quality_action(NewQualityAction { non_conformance_id: nc, action_type: "corrective".into(),
        procedure_id: None, description: "fix".into(), due_date: None }).await.unwrap();
    let a2 = svc.add_quality_action(NewQualityAction { non_conformance_id: nc, action_type: "preventive".into(),
        procedure_id: None, description: "prevent".into(), due_date: None }).await.unwrap();

    svc.complete_action(a1, dt("2026-07-07T10:00:00Z")).await.unwrap();
    assert!(matches!(svc.close_non_conformance(nc, dt("2026-07-07T10:30:00Z"), &sink).await, Err(QualityError::InvalidState(_))),
        "one action still open → close refused");
    svc.complete_action(a2, dt("2026-07-07T11:00:00Z")).await.unwrap();
    svc.close_non_conformance(nc, dt("2026-07-07T11:30:00Z"), &sink).await.unwrap();
    let st: String = sqlx::query_scalar("SELECT status::text FROM quality.non_conformances WHERE id=$1")
        .bind(nc).fetch_one(&pool).await.unwrap();
    assert_eq!(st, "closed");
}

/// IP-3 — no action can be added to a closed NC.
#[tokio::test]
async fn ip3_no_action_on_closed_nc() {
    let svc = QualityWriteService::new(pool().await);
    let sink = LoggingSink;
    let company = Uuid::new_v4();
    let nc = svc.raise_non_conformance(NewNonConformance {
        company_id: company, subject: "issue".into(), source_inspection_id: None, item_id: None,
        severity: "low".into(), description: None,
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    svc.close_non_conformance(nc, dt("2026-07-07T09:10:00Z"), &sink).await.unwrap(); // no actions → closeable

    let err = svc.add_quality_action(NewQualityAction { non_conformance_id: nc, action_type: "corrective".into(),
        procedure_id: None, description: "late".into(), due_date: None }).await.unwrap_err();
    assert!(matches!(err, QualityError::InvalidState(_)), "closed NC rejects new actions");
}

/// IP-4 — completing an action is idempotent (a retry is a no-op, not an error).
#[tokio::test]
async fn ip4_complete_action_idempotent() {
    let svc = QualityWriteService::new(pool().await);
    let sink = LoggingSink;
    let company = Uuid::new_v4();
    let nc = svc.raise_non_conformance(NewNonConformance {
        company_id: company, subject: "issue".into(), source_inspection_id: None, item_id: None,
        severity: "low".into(), description: None,
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    let a = svc.add_quality_action(NewQualityAction { non_conformance_id: nc, action_type: "corrective".into(),
        procedure_id: None, description: "fix".into(), due_date: None }).await.unwrap();
    svc.complete_action(a, dt("2026-07-07T10:00:00Z")).await.unwrap();
    svc.complete_action(a, dt("2026-07-07T11:00:00Z")).await.unwrap(); // no-op
}

/// IP-7 (completeness council 2026-07-07) — an in-process inspection produces a ROUTABLE disposition.
/// The brief keeps quality inspection "incoming/in-process"; the verdict core is trigger-agnostic, but
/// the disposition event carried no `inspection_type`, so a WIP (source-less) inspection emitted an event
/// a subscriber couldn't tell apart from an incoming-receipt one. The event now carries the type so a
/// future WIP/shipping consumer can route it (the incoming trigger+consumer is the shipped path; the
/// in-process trigger is deferred, ADR-001).
#[tokio::test]
async fn ip7_inspection_type_routable_on_the_event() {
    let svc = QualityWriteService::new(pool().await);
    let sink = CapturingSink::new();
    let (company, item) = (Uuid::new_v4(), Uuid::new_v4());
    let tpl = numeric_template(&svc, company).await;

    // An in-process (work-in-progress) check — no upstream Stock document.
    svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "in_process".into(),
        source_type: None, source_id: None, sample_size: 1,
        readings: vec![NewReading { parameter_name: "Diameter".into(), value: Some(dec("10.0")), manual_pass: None, remarks: None }],
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    // An incoming check on the same item.
    svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: Some("purchase_receipt".into()), source_id: Some(Uuid::new_v4()), sample_size: 1,
        readings: vec![NewReading { parameter_name: "Diameter".into(), value: Some(dec("10.0")), manual_pass: None, remarks: None }],
    }, dt("2026-07-07T09:05:00Z"), &sink).await.unwrap();

    let types: Vec<String> = sink.events().into_iter().filter_map(|e| match e {
        QualityEvent::QualityInspectionCompleted(c) => Some(c.inspection_type), _ => None }).collect();
    assert!(types.contains(&"in_process".to_string()), "the WIP disposition is routable as in_process");
    assert!(types.contains(&"incoming".to_string()), "the receipt disposition is routable as incoming");
    assert_ne!(types[0], types[1], "a subscriber can tell the two triggers apart");
}

/// IP-6 (maturity council 2026-07-07) — an inspection is ACCEPTED only if EVERY template parameter was
/// measured. `inspect` judged only the caller-supplied readings, so a template with two parameters
/// inspected with a single passing reading returned `accepted` — a false-pass releasing goods on the
/// unmeasured parameter. Now the full template must be covered: a missing (or duplicate/unknown) reading
/// is refused, never silently passed.
#[tokio::test]
async fn ip6_verdict_requires_full_parameter_coverage() {
    let svc = QualityWriteService::new(pool().await);
    let sink = LoggingSink;
    let (company, item) = (Uuid::new_v4(), Uuid::new_v4());
    // A two-parameter template: Diameter (numeric) + Finish (manual).
    let tpl = svc.create_template(NewTemplate {
        company_id: company, template_name: "Two-param".into(), item_id: None,
        parameters: vec![
            NewTemplateParameter { parameter_name: "Diameter".into(), numeric: true,
                min_value: Some(dec("9.5")), max_value: Some(dec("10.5")), spec_text: None },
            NewTemplateParameter { parameter_name: "Finish".into(), numeric: false,
                min_value: None, max_value: None, spec_text: Some("smooth".into()) },
        ],
    }).await.unwrap();

    // Only the passing Diameter is measured — Finish is skipped. Must NOT be accepted.
    let partial = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 1,
        readings: vec![NewReading { parameter_name: "Diameter".into(), value: Some(dec("10.0")), manual_pass: None, remarks: None }],
    }, dt("2026-07-07T09:00:00Z"), &sink).await;
    assert!(matches!(partial, Err(QualityError::Invalid(_))),
        "a template parameter with no reading must block the verdict, not pass unmeasured");

    // A duplicate reading for one parameter is refused too.
    let dup = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 1,
        readings: vec![
            NewReading { parameter_name: "Diameter".into(), value: Some(dec("10.0")), manual_pass: None, remarks: None },
            NewReading { parameter_name: "Diameter".into(), value: Some(dec("9.9")), manual_pass: None, remarks: None },
            NewReading { parameter_name: "Finish".into(), value: None, manual_pass: Some(true), remarks: None },
        ],
    }, dt("2026-07-07T09:00:00Z"), &sink).await;
    assert!(matches!(dup, Err(QualityError::Invalid(_))), "duplicate reading for a parameter is refused");

    // Full coverage passes.
    let full = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 1,
        readings: vec![
            NewReading { parameter_name: "Diameter".into(), value: Some(dec("10.0")), manual_pass: None, remarks: None },
            NewReading { parameter_name: "Finish".into(), value: None, manual_pass: Some(true), remarks: None },
        ],
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    assert!(full.accepted, "every parameter measured and passing → accepted");
}

/// IP-5 — a numeric reading with no measured value cannot be accepted (an unmeasured spec is a fail).
#[tokio::test]
async fn ip5_numeric_reading_needs_value() {
    let svc = QualityWriteService::new(pool().await);
    let sink = LoggingSink;
    let (company, item) = (Uuid::new_v4(), Uuid::new_v4());
    let tpl = numeric_template(&svc, company).await;
    let out = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 1,
        readings: vec![NewReading { parameter_name: "Diameter".into(), value: None, manual_pass: None, remarks: None }],
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    assert!(!out.accepted, "no measured value → rejected");
}
