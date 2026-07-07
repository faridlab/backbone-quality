use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use rust_decimal::Decimal;

use super::ReadingResult;
use super::AuditMetadata;

/// Strongly-typed ID for QualityInspectionReading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QualityInspectionReadingId(pub Uuid);

impl QualityInspectionReadingId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for QualityInspectionReadingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for QualityInspectionReadingId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for QualityInspectionReadingId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<QualityInspectionReadingId> for Uuid {
    fn from(id: QualityInspectionReadingId) -> Self { id.0 }
}

impl AsRef<Uuid> for QualityInspectionReadingId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for QualityInspectionReadingId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityInspectionReading {
    pub id: Uuid,
    pub inspection_id: Uuid,
    pub parameter_name: String,
    pub numeric: bool,
    pub reading_value: Option<Decimal>,
    pub min_value: Option<Decimal>,
    pub max_value: Option<Decimal>,
    pub manual_result: Option<bool>,
    pub result: ReadingResult,
    pub remarks: Option<String>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl QualityInspectionReading {
    /// Create a builder for QualityInspectionReading
    pub fn builder() -> QualityInspectionReadingBuilder {
        QualityInspectionReadingBuilder::default()
    }

    /// Create a new QualityInspectionReading with required fields
    pub fn new(inspection_id: Uuid, parameter_name: String, numeric: bool, result: ReadingResult) -> Self {
        Self {
            id: Uuid::new_v4(),
            inspection_id,
            parameter_name,
            numeric,
            reading_value: None,
            min_value: None,
            max_value: None,
            manual_result: None,
            result,
            remarks: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> QualityInspectionReadingId {
        QualityInspectionReadingId(self.id)
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


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the reading_value field (chainable)
    pub fn with_reading_value(mut self, value: Decimal) -> Self {
        self.reading_value = Some(value);
        self
    }

    /// Set the min_value field (chainable)
    pub fn with_min_value(mut self, value: Decimal) -> Self {
        self.min_value = Some(value);
        self
    }

    /// Set the max_value field (chainable)
    pub fn with_max_value(mut self, value: Decimal) -> Self {
        self.max_value = Some(value);
        self
    }

    /// Set the manual_result field (chainable)
    pub fn with_manual_result(mut self, value: bool) -> Self {
        self.manual_result = Some(value);
        self
    }

    /// Set the remarks field (chainable)
    pub fn with_remarks(mut self, value: String) -> Self {
        self.remarks = Some(value);
        self
    }

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "inspection_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.inspection_id = v; }
                }
                "parameter_name" => {
                    if let Ok(v) = serde_json::from_value(value) { self.parameter_name = v; }
                }
                "numeric" => {
                    if let Ok(v) = serde_json::from_value(value) { self.numeric = v; }
                }
                "reading_value" => {
                    if let Ok(v) = serde_json::from_value(value) { self.reading_value = v; }
                }
                "min_value" => {
                    if let Ok(v) = serde_json::from_value(value) { self.min_value = v; }
                }
                "max_value" => {
                    if let Ok(v) = serde_json::from_value(value) { self.max_value = v; }
                }
                "manual_result" => {
                    if let Ok(v) = serde_json::from_value(value) { self.manual_result = v; }
                }
                "result" => {
                    if let Ok(v) = serde_json::from_value(value) { self.result = v; }
                }
                "remarks" => {
                    if let Ok(v) = serde_json::from_value(value) { self.remarks = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for QualityInspectionReading {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "QualityInspectionReading"
    }
}

impl backbone_core::PersistentEntity for QualityInspectionReading {
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

impl backbone_orm::EntityRepoMeta for QualityInspectionReading {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("inspection_id".to_string(), "uuid".to_string());
        m.insert("result".to_string(), "reading_result".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["parameter_name"]
    }
}

/// Builder for QualityInspectionReading entity
///
/// Provides a fluent API for constructing QualityInspectionReading instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct QualityInspectionReadingBuilder {
    inspection_id: Option<Uuid>,
    parameter_name: Option<String>,
    numeric: Option<bool>,
    reading_value: Option<Decimal>,
    min_value: Option<Decimal>,
    max_value: Option<Decimal>,
    manual_result: Option<bool>,
    result: Option<ReadingResult>,
    remarks: Option<String>,
}

impl QualityInspectionReadingBuilder {
    /// Set the inspection_id field (required)
    pub fn inspection_id(mut self, value: Uuid) -> Self {
        self.inspection_id = Some(value);
        self
    }

    /// Set the parameter_name field (required)
    pub fn parameter_name(mut self, value: String) -> Self {
        self.parameter_name = Some(value);
        self
    }

    /// Set the numeric field (default: `true`)
    pub fn numeric(mut self, value: bool) -> Self {
        self.numeric = Some(value);
        self
    }

    /// Set the reading_value field (optional)
    pub fn reading_value(mut self, value: Decimal) -> Self {
        self.reading_value = Some(value);
        self
    }

    /// Set the min_value field (optional)
    pub fn min_value(mut self, value: Decimal) -> Self {
        self.min_value = Some(value);
        self
    }

    /// Set the max_value field (optional)
    pub fn max_value(mut self, value: Decimal) -> Self {
        self.max_value = Some(value);
        self
    }

    /// Set the manual_result field (optional)
    pub fn manual_result(mut self, value: bool) -> Self {
        self.manual_result = Some(value);
        self
    }

    /// Set the result field (default: `ReadingResult::default()`)
    pub fn result(mut self, value: ReadingResult) -> Self {
        self.result = Some(value);
        self
    }

    /// Set the remarks field (optional)
    pub fn remarks(mut self, value: String) -> Self {
        self.remarks = Some(value);
        self
    }

    /// Build the QualityInspectionReading entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<QualityInspectionReading, String> {
        let inspection_id = self.inspection_id.ok_or_else(|| "inspection_id is required".to_string())?;
        let parameter_name = self.parameter_name.ok_or_else(|| "parameter_name is required".to_string())?;

        Ok(QualityInspectionReading {
            id: Uuid::new_v4(),
            inspection_id,
            parameter_name,
            numeric: self.numeric.unwrap_or(true),
            reading_value: self.reading_value,
            min_value: self.min_value,
            max_value: self.max_value,
            manual_result: self.manual_result,
            result: self.result.unwrap_or(ReadingResult::default()),
            remarks: self.remarks,
            metadata: AuditMetadata::default(),
        })
    }
}
