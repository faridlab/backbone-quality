//! Shared test helpers: a live pool + an event-capturing sink + fixed-timestamp / decimal builders.
//! Quality drives no neighbour, so there is no fake port — the seam reads a REAL backbone-inventory
//! Purchase Receipt directly. Fresh random ids per test.

#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use backbone_quality::application::service::quality_events::{QualityEvent, QualityEventSink};
pub use backbone_quality::application::service::quality_events::LoggingSink;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

/// Live-DB tests require an explicit `DATABASE_URL`. When it is unset, [`pool`]
/// returns `None` and each test skips gracefully — matching the convention in
/// `backbone-orm/tests/rls_scope_live.rs`. CI always sets it, so the tests run
/// and gate there; local runs without a Postgres stay clean instead of panicking.
pub fn dburl() -> String {
    std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run live-DB tests (call sites guard with pool())")
}
/// Returns `None` (skip) when no `DATABASE_URL` is configured; otherwise connects
/// and returns `Some(pool)`. A connect failure with a DSN present is a real
/// problem, so it `.expect`s — fail loud, never false-green.
pub async fn pool() -> Option<PgPool> {
    if std::env::var("DATABASE_URL").is_err() {
        eprintln!("skipping live-DB test: DATABASE_URL not set");
        return None;
    }
    Some(PgPool::connect(&dburl()).await.expect("connect"))
}
pub fn dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}
pub fn dec(s: &str) -> Decimal {
    s.parse().unwrap()
}

/// A sink that records every published quality event.
#[derive(Clone, Default)]
pub struct CapturingSink {
    pub events: Arc<Mutex<Vec<QualityEvent>>>,
}
impl CapturingSink {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn events(&self) -> Vec<QualityEvent> {
        self.events.lock().unwrap().clone()
    }
}
impl QualityEventSink for CapturingSink {
    fn publish(&self, event: &QualityEvent) {
        self.events.lock().unwrap().push(event.clone());
    }
}
