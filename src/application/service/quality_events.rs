//! Quality domain events (hand-authored, user-owned) — the public extension surface.
//!
//! backbone-quality posts NO GL and drives no neighbour. Its coupling is **outbound events + inbound
//! logical reads** (brief §5.3): the inspection disposition (`QualityInspectionCompleted`, carrying
//! accepted/rejected) is the signal Stock subscribes to ("accept into stock / hold"), and the CAPA
//! lifecycle (`NonConformanceRaised` / `NonConformanceClosed`) is a read-side quality signal. A consuming
//! service supplies the sink (bus, outbox, …).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An inspection reached its verdict — `accepted` is the disposition Stock acts on.
///
/// `inspection_type` (incoming / in_process / outgoing) lets a subscriber ROUTE the disposition by
/// trigger: an `incoming` verdict is the accept/hold signal Stock correlates on `source_id`; an
/// `in_process` / `outgoing` verdict is for a future WIP/shipping consumer. Only the incoming path has a
/// wired trigger + consumer today (a manufacturing trigger is deferred, ADR-001) — but the event carries
/// the type so that path is additive, not a breaking change.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityInspectionCompleted {
    pub inspection_id: Uuid,
    pub company_id: Uuid,
    pub item_id: Uuid,
    pub inspection_type: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub accepted: bool,
}

/// A non-conformance was raised (from a rejected inspection or an audit).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NonConformanceRaised {
    pub non_conformance_id: Uuid,
    pub company_id: Uuid,
    pub source_inspection_id: Option<Uuid>,
    pub severity: String,
}

/// A non-conformance was closed (all its actions completed).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NonConformanceClosed {
    pub non_conformance_id: Uuid,
    pub company_id: Uuid,
}

/// The quality domain-event union.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum QualityEvent {
    QualityInspectionCompleted(QualityInspectionCompleted),
    NonConformanceRaised(NonConformanceRaised),
    NonConformanceClosed(NonConformanceClosed),
}

/// Sink the write path publishes to. A consuming service supplies its own (bus, outbox, …).
pub trait QualityEventSink: Send + Sync {
    fn publish(&self, event: &QualityEvent);
}

/// A no-op/logging sink for tests and single-process composition.
#[derive(Debug, Default, Clone)]
pub struct LoggingSink;

impl QualityEventSink for LoggingSink {
    fn publish(&self, event: &QualityEvent) {
        tracing::info!(?event, "quality event");
    }
}
