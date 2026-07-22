use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::InspectionType;
use super::InspectionStatus;
use super::AuditMetadata;

/// Strongly-typed ID for QualityInspection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QualityInspectionId(pub Uuid);

impl QualityInspectionId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for QualityInspectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for QualityInspectionId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for QualityInspectionId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<QualityInspectionId> for Uuid {
    fn from(id: QualityInspectionId) -> Self { id.0 }
}

impl AsRef<Uuid> for QualityInspectionId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for QualityInspectionId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityInspection {
    pub id: Uuid,
    pub company_id: Uuid,
    pub template_id: Option<Uuid>,
    pub item_id: Uuid,
    pub inspection_type: InspectionType,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub sample_size: i32,
    pub inspected_at: DateTime<Utc>,
    pub status: InspectionStatus,
    pub remarks: Option<String>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl QualityInspection {
    /// Create a builder for QualityInspection
    pub fn builder() -> QualityInspectionBuilder {
        QualityInspectionBuilder::default()
    }

    /// Create a new QualityInspection with required fields
    pub fn new(company_id: Uuid, item_id: Uuid, inspection_type: InspectionType, sample_size: i32, inspected_at: DateTime<Utc>, status: InspectionStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            template_id: None,
            item_id,
            inspection_type,
            source_type: None,
            source_id: None,
            sample_size,
            inspected_at,
            status,
            remarks: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> QualityInspectionId {
        QualityInspectionId(self.id)
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
    pub fn status(&self) -> &InspectionStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the template_id field (chainable)
    pub fn with_template_id(mut self, value: Uuid) -> Self {
        self.template_id = Some(value);
        self
    }

    /// Set the source_type field (chainable)
    pub fn with_source_type(mut self, value: String) -> Self {
        self.source_type = Some(value);
        self
    }

    /// Set the source_id field (chainable)
    pub fn with_source_id(mut self, value: Uuid) -> Self {
        self.source_id = Some(value);
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
                "company_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.company_id = v; }
                }
                "template_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.template_id = v; }
                }
                "item_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.item_id = v; }
                }
                "inspection_type" => {
                    if let Ok(v) = serde_json::from_value(value) { self.inspection_type = v; }
                }
                "source_type" => {
                    if let Ok(v) = serde_json::from_value(value) { self.source_type = v; }
                }
                "source_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.source_id = v; }
                }
                "sample_size" => {
                    if let Ok(v) = serde_json::from_value(value) { self.sample_size = v; }
                }
                "inspected_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.inspected_at = v; }
                }
                "status" => {
                    if let Ok(v) = serde_json::from_value(value) { self.status = v; }
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

impl super::Entity for QualityInspection {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "QualityInspection"
    }
}

impl backbone_core::PersistentEntity for QualityInspection {
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

impl backbone_orm::EntityRepoMeta for QualityInspection {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("company_id".to_string(), "uuid".to_string());
        m.insert("template_id".to_string(), "uuid".to_string());
        m.insert("item_id".to_string(), "uuid".to_string());
        m.insert("source_id".to_string(), "uuid".to_string());
        m.insert("inspection_type".to_string(), "inspection_type".to_string());
        m.insert("status".to_string(), "inspection_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &[]
    }
    fn company_field() -> Option<&'static str> {
        Some("company_id")
    }
}

/// Builder for QualityInspection entity
///
/// Provides a fluent API for constructing QualityInspection instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct QualityInspectionBuilder {
    company_id: Option<Uuid>,
    template_id: Option<Uuid>,
    item_id: Option<Uuid>,
    inspection_type: Option<InspectionType>,
    source_type: Option<String>,
    source_id: Option<Uuid>,
    sample_size: Option<i32>,
    inspected_at: Option<DateTime<Utc>>,
    status: Option<InspectionStatus>,
    remarks: Option<String>,
}

impl QualityInspectionBuilder {
    /// Set the company_id field (required)
    pub fn company_id(mut self, value: Uuid) -> Self {
        self.company_id = Some(value);
        self
    }

    /// Set the template_id field (optional)
    pub fn template_id(mut self, value: Uuid) -> Self {
        self.template_id = Some(value);
        self
    }

    /// Set the item_id field (required)
    pub fn item_id(mut self, value: Uuid) -> Self {
        self.item_id = Some(value);
        self
    }

    /// Set the inspection_type field (default: `InspectionType::default()`)
    pub fn inspection_type(mut self, value: InspectionType) -> Self {
        self.inspection_type = Some(value);
        self
    }

    /// Set the source_type field (optional)
    pub fn source_type(mut self, value: String) -> Self {
        self.source_type = Some(value);
        self
    }

    /// Set the source_id field (optional)
    pub fn source_id(mut self, value: Uuid) -> Self {
        self.source_id = Some(value);
        self
    }

    /// Set the sample_size field (default: `1`)
    pub fn sample_size(mut self, value: i32) -> Self {
        self.sample_size = Some(value);
        self
    }

    /// Set the inspected_at field (required)
    pub fn inspected_at(mut self, value: DateTime<Utc>) -> Self {
        self.inspected_at = Some(value);
        self
    }

    /// Set the status field (default: `InspectionStatus::default()`)
    pub fn status(mut self, value: InspectionStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set the remarks field (optional)
    pub fn remarks(mut self, value: String) -> Self {
        self.remarks = Some(value);
        self
    }

    /// Build the QualityInspection entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<QualityInspection, String> {
        let company_id = self.company_id.ok_or_else(|| "company_id is required".to_string())?;
        let item_id = self.item_id.ok_or_else(|| "item_id is required".to_string())?;
        let inspected_at = self.inspected_at.ok_or_else(|| "inspected_at is required".to_string())?;

        Ok(QualityInspection {
            id: Uuid::new_v4(),
            company_id,
            template_id: self.template_id,
            item_id,
            inspection_type: self.inspection_type.unwrap_or(InspectionType::default()),
            source_type: self.source_type,
            source_id: self.source_id,
            sample_size: self.sample_size.unwrap_or(1),
            inspected_at,
            status: self.status.unwrap_or(InspectionStatus::default()),
            remarks: self.remarks,
            metadata: AuditMetadata::default(),
        })
    }
}
