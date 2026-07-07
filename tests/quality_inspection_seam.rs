//! The inspection-trigger seam, end-to-end: **backbone-quality reads a REAL backbone-inventory Purchase
//! Receipt**.
//!   QSEAM-1 (Purchase Receipt → incoming inspection): a real `inventory.purchase_receipts` row (the
//!            Stock document that triggers an incoming inspection) is created, then `inspect` records an
//!            inspection linked to it (`source_type='purchase_receipt'`, `source_id=<real receipt>`) and
//!            emits the disposition event carrying that source, so Stock can correlate accept/hold.
//! This edge is a dev-dependency ONLY — the shipped quality library depends on neither inventory nor GL;
//! quality drives nothing (brief §5.3: accept/reject is an EVENT Stock subscribes to).

mod common;

use backbone_quality::application::service::quality_events::QualityEvent;
use backbone_quality::application::service::quality_write_service::{
    NewInspection, NewReading, NewTemplate, NewTemplateParameter, QualityWriteService,
};
use common::*;
use uuid::Uuid;

/// QSEAM-1 — an incoming inspection links a REAL upstream Purchase Receipt and signals the disposition.
#[tokio::test]
async fn qseam1_inspect_a_real_purchase_receipt() {
    let pool = pool().await;
    let quality = QualityWriteService::new(pool.clone());
    let inventory = backbone_inventory::application::service::inventory_write_service::InventoryWriteService::new(pool.clone());
    let sink = CapturingSink::new();
    let company = Uuid::new_v4();
    let item = Uuid::new_v4();

    // A REAL inbound Stock document: a purchase receipt (draft) with the item that will be inspected.
    use backbone_inventory::application::service::inventory_write_service::{NewReceipt, ReceiptLine};
    let receipt = inventory.create_purchase_receipt(NewReceipt {
        receipt_number: format!("PR-{}", Uuid::new_v4()),
        company_id: company, branch_id: None, supplier_id: Uuid::new_v4(), source_po_id: None,
        warehouse_id: Uuid::new_v4(), posting_date: chrono::Utc::now().date_naive(),
        inventory_account_id: Uuid::new_v4(), grir_account_id: Uuid::new_v4(),
        lines: vec![ReceiptLine { item_id: item, quantity: dec("100"), rate: dec("2500") }],
    }).await.unwrap();

    // The received item is the item on the receipt line.
    let recv_item: Uuid = sqlx::query_scalar(
        "SELECT item_id FROM inventory.purchase_receipt_items WHERE receipt_id=$1 LIMIT 1")
        .bind(receipt).fetch_one(&pool).await.unwrap();
    assert_eq!(recv_item, item);

    // Incoming inspection triggered by (and linked to) the real receipt.
    let tpl = quality.create_template(NewTemplate {
        company_id: company, template_name: "Incoming QC".into(), item_id: Some(item),
        parameters: vec![NewTemplateParameter { parameter_name: "Moisture".into(), numeric: true,
            min_value: None, max_value: Some(dec("12.0")), spec_text: None }],
    }).await.unwrap();
    let out = quality.inspect(NewInspection {
        company_id: company, template_id: tpl, item_id: recv_item, inspection_type: "incoming".into(),
        source_type: Some("purchase_receipt".into()), source_id: Some(receipt), sample_size: 10,
        readings: vec![NewReading { parameter_name: "Moisture".into(), value: Some(dec("9.4")), manual_pass: None, remarks: None }],
    }, dt("2026-07-07T09:00:00Z"), &sink).await.unwrap();
    assert!(out.accepted);

    // The inspection links the real receipt.
    let (stype, sid): (Option<String>, Option<Uuid>) = sqlx::query_as(
        "SELECT source_type, source_id FROM quality.quality_inspections WHERE id=$1")
        .bind(out.inspection_id).fetch_one(&pool).await.unwrap();
    assert_eq!(stype.as_deref(), Some("purchase_receipt"));
    assert_eq!(sid, Some(receipt), "inspection is linked to the real purchase receipt");

    // The disposition event carries the source so Stock can accept/hold the right receipt.
    let ev = sink.events().into_iter().find_map(|e| match e {
        QualityEvent::QualityInspectionCompleted(c) => Some(c), _ => None }).unwrap();
    assert_eq!(ev.source_id, Some(receipt));
    assert!(ev.accepted, "in-spec incoming goods signal ACCEPT into stock");
}
