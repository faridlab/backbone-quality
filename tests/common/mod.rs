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

pub fn dburl() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/backbone_quality".into())
}
pub async fn pool() -> PgPool {
    PgPool::connect(&dburl()).await.expect("connect")
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
