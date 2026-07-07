use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use super::AuditMetadata;

/// Strongly-typed ID for QualityProcedure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QualityProcedureId(pub Uuid);

impl QualityProcedureId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for QualityProcedureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for QualityProcedureId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for QualityProcedureId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<QualityProcedureId> for Uuid {
    fn from(id: QualityProcedureId) -> Self { id.0 }
}

impl AsRef<Uuid> for QualityProcedureId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for QualityProcedureId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityProcedure {
    pub id: Uuid,
    pub company_id: Uuid,
    pub procedure_name: String,
    pub parent_procedure_id: Option<Uuid>,
    pub description: Option<String>,
    pub is_active: bool,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl QualityProcedure {
    /// Create a builder for QualityProcedure
    pub fn builder() -> QualityProcedureBuilder {
        QualityProcedureBuilder::default()
    }

    /// Create a new QualityProcedure with required fields
    pub fn new(company_id: Uuid, procedure_name: String, is_active: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            procedure_name,
            parent_procedure_id: None,
            description: None,
            is_active,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> QualityProcedureId {
        QualityProcedureId(self.id)
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

    /// Set the parent_procedure_id field (chainable)
    pub fn with_parent_procedure_id(mut self, value: Uuid) -> Self {
        self.parent_procedure_id = Some(value);
        self
    }

    /// Set the description field (chainable)
    pub fn with_description(mut self, value: String) -> Self {
        self.description = Some(value);
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
                "procedure_name" => {
                    if let Ok(v) = serde_json::from_value(value) { self.procedure_name = v; }
                }
                "parent_procedure_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.parent_procedure_id = v; }
                }
                "description" => {
                    if let Ok(v) = serde_json::from_value(value) { self.description = v; }
                }
                "is_active" => {
                    if let Ok(v) = serde_json::from_value(value) { self.is_active = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for QualityProcedure {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "QualityProcedure"
    }
}

impl backbone_core::PersistentEntity for QualityProcedure {
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

impl backbone_orm::EntityRepoMeta for QualityProcedure {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("company_id".to_string(), "uuid".to_string());
        m.insert("parent_procedure_id".to_string(), "uuid".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["procedure_name"]
    }
}

/// Builder for QualityProcedure entity
///
/// Provides a fluent API for constructing QualityProcedure instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct QualityProcedureBuilder {
    company_id: Option<Uuid>,
    procedure_name: Option<String>,
    parent_procedure_id: Option<Uuid>,
    description: Option<String>,
    is_active: Option<bool>,
}

impl QualityProcedureBuilder {
    /// Set the company_id field (required)
    pub fn company_id(mut self, value: Uuid) -> Self {
        self.company_id = Some(value);
        self
    }

    /// Set the procedure_name field (required)
    pub fn procedure_name(mut self, value: String) -> Self {
        self.procedure_name = Some(value);
        self
    }

    /// Set the parent_procedure_id field (optional)
    pub fn parent_procedure_id(mut self, value: Uuid) -> Self {
        self.parent_procedure_id = Some(value);
        self
    }

    /// Set the description field (optional)
    pub fn description(mut self, value: String) -> Self {
        self.description = Some(value);
        self
    }

    /// Set the is_active field (default: `true`)
    pub fn is_active(mut self, value: bool) -> Self {
        self.is_active = Some(value);
        self
    }

    /// Build the QualityProcedure entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<QualityProcedure, String> {
        let company_id = self.company_id.ok_or_else(|| "company_id is required".to_string())?;
        let procedure_name = self.procedure_name.ok_or_else(|| "procedure_name is required".to_string())?;

        Ok(QualityProcedure {
            id: Uuid::new_v4(),
            company_id,
            procedure_name,
            parent_procedure_id: self.parent_procedure_id,
            description: self.description,
            is_active: self.is_active.unwrap_or(true),
            metadata: AuditMetadata::default(),
        })
    }
}
