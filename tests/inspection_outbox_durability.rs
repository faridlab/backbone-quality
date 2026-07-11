//! Durability probe (outbox rollout plan, P1): the cross-module `QualityInspectionCompleted` disposition —
//! which Stock SUBSCRIBES to, to accept/reject the inspected lot — is staged in the transactional outbox in
//! the SAME tx as the inspection, so a crash between commit and the in-proc publish cannot strand goods.
//! The `LoggingSink` drops the in-proc publish; the disposition must still be staged in
//! `quality.outbox_events` for the relay to drain.

mod common;

use backbone_quality::application::service::quality_events::LoggingSink;
use backbone_quality::application::service::quality_write_service::{
    NewInspection, NewReading, NewTemplate, NewTemplateParameter, QualityWriteService,
};
use common::*;
use uuid::Uuid;

// QOD-1 — an inspection durably stages its disposition despite the dropped in-proc publish.
#[tokio::test]
async fn qod1_disposition_is_durably_staged() {
    let pool = pool().await;
    let svc = QualityWriteService::new(pool.clone());
    let (company, item) = (Uuid::new_v4(), Uuid::new_v4());
    let tpl = svc.create_template(NewTemplate {
        company_id: company, template_name: "Widget QC".into(), item_id: Some(item),
        parameters: vec![NewTemplateParameter {
            parameter_name: "Diameter".into(), numeric: true,
            min_value: Some(dec("9.5")), max_value: Some(dec("10.5")), spec_text: None }],
    }).await.unwrap();

    // LoggingSink drops the in-proc publish — the durability must come from the outbox, not the sink.
    let out = svc.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: item, inspection_type: "incoming".into(),
        source_type: None, source_id: None, sample_size: 5,
        readings: vec![NewReading {
            parameter_name: "Diameter".into(), value: Some(dec("10.0")), manual_pass: None, remarks: None }],
    }, dt("2026-07-07T09:00:00Z"), &LoggingSink).await.unwrap();

    let staged: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM quality.outbox_events WHERE aggregate_id=$1 AND event_type='QualityInspectionCompleted'")
        .bind(out.inspection_id.to_string()).fetch_one(&pool).await.unwrap();
    assert_eq!(staged, 1, "QualityInspectionCompleted durably staged despite the dropped in-proc publish");
}
