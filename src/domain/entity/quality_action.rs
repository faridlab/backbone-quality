use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::QualityActionType;
use super::QualityActionStatus;
use super::AuditMetadata;

/// Strongly-typed ID for QualityAction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QualityActionId(pub Uuid);

impl QualityActionId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for QualityActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for QualityActionId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for QualityActionId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<QualityActionId> for Uuid {
    fn from(id: QualityActionId) -> Self { id.0 }
}

impl AsRef<Uuid> for QualityActionId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for QualityActionId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityAction {
    pub id: Uuid,
    pub company_id: Uuid,
    pub non_conformance_id: Uuid,
    pub action_type: QualityActionType,
    pub procedure_id: Option<Uuid>,
    pub status: QualityActionStatus,
    pub description: String,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl QualityAction {
    /// Create a builder for QualityAction
    pub fn builder() -> QualityActionBuilder {
        QualityActionBuilder::default()
    }

    /// Create a new QualityAction with required fields
    pub fn new(company_id: Uuid, non_conformance_id: Uuid, action_type: QualityActionType, status: QualityActionStatus, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            non_conformance_id,
            action_type,
            procedure_id: None,
            status,
            description,
            due_date: None,
            completed_at: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> QualityActionId {
        QualityActionId(self.id)
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
    pub fn status(&self) -> &QualityActionStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the procedure_id field (chainable)
    pub fn with_procedure_id(mut self, value: Uuid) -> Self {
        self.procedure_id = Some(value);
        self
    }

    /// Set the due_date field (chainable)
    pub fn with_due_date(mut self, value: DateTime<Utc>) -> Self {
        self.due_date = Some(value);
        self
    }

    /// Set the completed_at field (chainable)
    pub fn with_completed_at(mut self, value: DateTime<Utc>) -> Self {
        self.completed_at = Some(value);
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
                "non_conformance_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.non_conformance_id = v; }
                }
                "action_type" => {
                    if let Ok(v) = serde_json::from_value(value) { self.action_type = v; }
                }
                "procedure_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.procedure_id = v; }
                }
                "status" => {
                    if let Ok(v) = serde_json::from_value(value) { self.status = v; }
                }
                "description" => {
                    if let Ok(v) = serde_json::from_value(value) { self.description = v; }
                }
                "due_date" => {
                    if let Ok(v) = serde_json::from_value(value) { self.due_date = v; }
                }
                "completed_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.completed_at = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for QualityAction {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "QualityAction"
    }
}

impl backbone_core::PersistentEntity for QualityAction {
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

impl backbone_orm::EntityRepoMeta for QualityAction {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("company_id".to_string(), "uuid".to_string());
        m.insert("non_conformance_id".to_string(), "uuid".to_string());
        m.insert("procedure_id".to_string(), "uuid".to_string());
        m.insert("action_type".to_string(), "quality_action_type".to_string());
        m.insert("status".to_string(), "quality_action_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["description"]
    }
}

/// Builder for QualityAction entity
///
/// Provides a fluent API for constructing QualityAction instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct QualityActionBuilder {
    company_id: Option<Uuid>,
    non_conformance_id: Option<Uuid>,
    action_type: Option<QualityActionType>,
    procedure_id: Option<Uuid>,
    status: Option<QualityActionStatus>,
    description: Option<String>,
    due_date: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

impl QualityActionBuilder {
    /// Set the company_id field (required)
    pub fn company_id(mut self, value: Uuid) -> Self {
        self.company_id = Some(value);
        self
    }

    /// Set the non_conformance_id field (required)
    pub fn non_conformance_id(mut self, value: Uuid) -> Self {
        self.non_conformance_id = Some(value);
        self
    }

    /// Set the action_type field (default: `QualityActionType::default()`)
    pub fn action_type(mut self, value: QualityActionType) -> Self {
        self.action_type = Some(value);
        self
    }

    /// Set the procedure_id field (optional)
    pub fn procedure_id(mut self, value: Uuid) -> Self {
        self.procedure_id = Some(value);
        self
    }

    /// Set the status field (default: `QualityActionStatus::default()`)
    pub fn status(mut self, value: QualityActionStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set the description field (required)
    pub fn description(mut self, value: String) -> Self {
        self.description = Some(value);
        self
    }

    /// Set the due_date field (optional)
    pub fn due_date(mut self, value: DateTime<Utc>) -> Self {
        self.due_date = Some(value);
        self
    }

    /// Set the completed_at field (optional)
    pub fn completed_at(mut self, value: DateTime<Utc>) -> Self {
        self.completed_at = Some(value);
        self
    }

    /// Build the QualityAction entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<QualityAction, String> {
        let company_id = self.company_id.ok_or_else(|| "company_id is required".to_string())?;
        let non_conformance_id = self.non_conformance_id.ok_or_else(|| "non_conformance_id is required".to_string())?;
        let description = self.description.ok_or_else(|| "description is required".to_string())?;

        Ok(QualityAction {
            id: Uuid::new_v4(),
            company_id,
            non_conformance_id,
            action_type: self.action_type.unwrap_or(QualityActionType::default()),
            procedure_id: self.procedure_id,
            status: self.status.unwrap_or(QualityActionStatus::default()),
            description,
            due_date: self.due_date,
            completed_at: self.completed_at,
            metadata: AuditMetadata::default(),
        })
    }
}
