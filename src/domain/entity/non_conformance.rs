use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::NonConformanceSeverity;
use super::NonConformanceStatus;
use super::AuditMetadata;

/// Strongly-typed ID for NonConformance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NonConformanceId(pub Uuid);

impl NonConformanceId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for NonConformanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for NonConformanceId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for NonConformanceId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<NonConformanceId> for Uuid {
    fn from(id: NonConformanceId) -> Self { id.0 }
}

impl AsRef<Uuid> for NonConformanceId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for NonConformanceId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NonConformance {
    pub id: Uuid,
    pub company_id: Uuid,
    pub subject: String,
    pub source_inspection_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub severity: NonConformanceSeverity,
    pub status: NonConformanceStatus,
    pub description: Option<String>,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl NonConformance {
    /// Create a builder for NonConformance
    pub fn builder() -> NonConformanceBuilder {
        NonConformanceBuilder::default()
    }

    /// Create a new NonConformance with required fields
    pub fn new(company_id: Uuid, subject: String, severity: NonConformanceSeverity, status: NonConformanceStatus, opened_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            subject,
            source_inspection_id: None,
            item_id: None,
            severity,
            status,
            description: None,
            opened_at,
            closed_at: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> NonConformanceId {
        NonConformanceId(self.id)
    }

    /// Get when this entity was created
    pub fn created_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.created_at.as_ref()
    }

    /// Get when this entity was last updated
    pub fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.updated_at.as_ref()
    }

    /// Check if this entity is soft deleted
    pub fn is_deleted(&self) -> bool {
        self.metadata.deleted_at.is_some()
    }

    /// Check if this entity is active (not deleted)
    pub fn is_active(&self) -> bool {
        self.metadata.deleted_at.is_none()
    }

    /// Get when this entity was deleted
    pub fn deleted_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.deleted_at.as_ref()
    }

    /// Get who created this entity
    pub fn created_by(&self) -> Option<&Uuid> {
        self.metadata.created_by.as_ref()
    }

    /// Get who last updated this entity
    pub fn updated_by(&self) -> Option<&Uuid> {
        self.metadata.updated_by.as_ref()
    }

    /// Get who deleted this entity
    pub fn deleted_by(&self) -> Option<&Uuid> {
        self.metadata.deleted_by.as_ref()
    }

    /// Get the current status
    pub fn status(&self) -> &NonConformanceStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the source_inspection_id field (chainable)
    pub fn with_source_inspection_id(mut self, value: Uuid) -> Self {
        self.source_inspection_id = Some(value);
        self
    }

    /// Set the item_id field (chainable)
    pub fn with_item_id(mut self, value: Uuid) -> Self {
        self.item_id = Some(value);
        self
    }

    /// Set the description field (chainable)
    pub fn with_description(mut self, value: String) -> Self {
        self.description = Some(value);
        self
    }

    /// Set the closed_at field (chainable)
    pub fn with_closed_at(mut self, value: DateTime<Utc>) -> Self {
        self.closed_at = Some(value);
        self
    }

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "company_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.company_id = v; }
                }
                "subject" => {
                    if let Ok(v) = serde_json::from_value(value) { self.subject = v; }
                }
                "source_inspection_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.source_inspection_id = v; }
                }
                "item_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.item_id = v; }
                }
                "severity" => {
                    if let Ok(v) = serde_json::from_value(value) { self.severity = v; }
                }
                "status" => {
                    if let Ok(v) = serde_json::from_value(value) { self.status = v; }
                }
                "description" => {
                    if let Ok(v) = serde_json::from_value(value) { self.description = v; }
                }
                "opened_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.opened_at = v; }
                }
                "closed_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.closed_at = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for NonConformance {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "NonConformance"
    }
}

impl backbone_core::PersistentEntity for NonConformance {
    fn entity_id(&self) -> String {
        self.id.to_string()
    }
    fn set_entity_id(&mut self, id: String) {
        if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
            self.id = uuid;
        }
    }
    fn created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.created_at
    }
    fn set_created_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
        self.metadata.created_at = Some(ts);
    }
    fn updated_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.updated_at
    }
    fn set_updated_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
        self.metadata.updated_at = Some(ts);
    }
    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.deleted_at
    }
    fn set_deleted_at(&mut self, ts: Option<chrono::DateTime<chrono::Utc>>) {
        self.metadata.deleted_at = ts;
    }
}

impl backbone_orm::EntityRepoMeta for NonConformance {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("company_id".to_string(), "uuid".to_string());
        m.insert("source_inspection_id".to_string(), "uuid".to_string());
        m.insert("item_id".to_string(), "uuid".to_string());
        m.insert("severity".to_string(), "non_conformance_severity".to_string());
        m.insert("status".to_string(), "non_conformance_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["subject"]
    }
}

/// Builder for NonConformance entity
///
/// Provides a fluent API for constructing NonConformance instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct NonConformanceBuilder {
    company_id: Option<Uuid>,
    subject: Option<String>,
    source_inspection_id: Option<Uuid>,
    item_id: Option<Uuid>,
    severity: Option<NonConformanceSeverity>,
    status: Option<NonConformanceStatus>,
    description: Option<String>,
    opened_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
}

impl NonConformanceBuilder {
    /// Set the company_id field (required)
    pub fn company_id(mut self, value: Uuid) -> Self {
        self.company_id = Some(value);
        self
    }

    /// Set the subject field (required)
    pub fn subject(mut self, value: String) -> Self {
        self.subject = Some(value);
        self
    }

    /// Set the source_inspection_id field (optional)
    pub fn source_inspection_id(mut self, value: Uuid) -> Self {
        self.source_inspection_id = Some(value);
        self
    }

    /// Set the item_id field (optional)
    pub fn item_id(mut self, value: Uuid) -> Self {
        self.item_id = Some(value);
        self
    }

    /// Set the severity field (default: `NonConformanceSeverity::default()`)
    pub fn severity(mut self, value: NonConformanceSeverity) -> Self {
        self.severity = Some(value);
        self
    }

    /// Set the status field (default: `NonConformanceStatus::default()`)
    pub fn status(mut self, value: NonConformanceStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set the description field (optional)
    pub fn description(mut self, value: String) -> Self {
        self.description = Some(value);
        self
    }

    /// Set the opened_at field (required)
    pub fn opened_at(mut self, value: DateTime<Utc>) -> Self {
        self.opened_at = Some(value);
        self
    }

    /// Set the closed_at field (optional)
    pub fn closed_at(mut self, value: DateTime<Utc>) -> Self {
        self.closed_at = Some(value);
        self
    }

    /// Build the NonConformance entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<NonConformance, String> {
        let company_id = self.company_id.ok_or_else(|| "company_id is required".to_string())?;
        let subject = self.subject.ok_or_else(|| "subject is required".to_string())?;
        let opened_at = self.opened_at.ok_or_else(|| "opened_at is required".to_string())?;

        Ok(NonConformance {
            id: Uuid::new_v4(),
            company_id,
            subject,
            source_inspection_id: self.source_inspection_id,
            item_id: self.item_id,
            severity: self.severity.unwrap_or(NonConformanceSeverity::default()),
            status: self.status.unwrap_or(NonConformanceStatus::default()),
            description: self.description,
            opened_at,
            closed_at: self.closed_at,
            metadata: AuditMetadata::default(),
        })
    }
}
