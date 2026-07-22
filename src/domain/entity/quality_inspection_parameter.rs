use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use rust_decimal::Decimal;
use super::AuditMetadata;

/// Strongly-typed ID for QualityInspectionParameter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QualityInspectionParameterId(pub Uuid);

impl QualityInspectionParameterId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for QualityInspectionParameterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for QualityInspectionParameterId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for QualityInspectionParameterId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<QualityInspectionParameterId> for Uuid {
    fn from(id: QualityInspectionParameterId) -> Self { id.0 }
}

impl AsRef<Uuid> for QualityInspectionParameterId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for QualityInspectionParameterId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityInspectionParameter {
    pub id: Uuid,
    pub company_id: Uuid,
    pub template_id: Uuid,
    pub parameter_name: String,
    pub numeric: bool,
    pub min_value: Option<Decimal>,
    pub max_value: Option<Decimal>,
    pub spec_text: Option<String>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl QualityInspectionParameter {
    /// Create a builder for QualityInspectionParameter
    pub fn builder() -> QualityInspectionParameterBuilder {
        QualityInspectionParameterBuilder::default()
    }

    /// Create a new QualityInspectionParameter with required fields
    pub fn new(company_id: Uuid, template_id: Uuid, parameter_name: String, numeric: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            template_id,
            parameter_name,
            numeric,
            min_value: None,
            max_value: None,
            spec_text: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> QualityInspectionParameterId {
        QualityInspectionParameterId(self.id)
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

    /// Set the spec_text field (chainable)
    pub fn with_spec_text(mut self, value: String) -> Self {
        self.spec_text = Some(value);
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
                "parameter_name" => {
                    if let Ok(v) = serde_json::from_value(value) { self.parameter_name = v; }
                }
                "numeric" => {
                    if let Ok(v) = serde_json::from_value(value) { self.numeric = v; }
                }
                "min_value" => {
                    if let Ok(v) = serde_json::from_value(value) { self.min_value = v; }
                }
                "max_value" => {
                    if let Ok(v) = serde_json::from_value(value) { self.max_value = v; }
                }
                "spec_text" => {
                    if let Ok(v) = serde_json::from_value(value) { self.spec_text = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for QualityInspectionParameter {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "QualityInspectionParameter"
    }
}

impl backbone_core::PersistentEntity for QualityInspectionParameter {
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

impl backbone_orm::EntityRepoMeta for QualityInspectionParameter {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("company_id".to_string(), "uuid".to_string());
        m.insert("template_id".to_string(), "uuid".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["parameter_name"]
    }
    fn company_field() -> Option<&'static str> {
        Some("company_id")
    }
}

/// Builder for QualityInspectionParameter entity
///
/// Provides a fluent API for constructing QualityInspectionParameter instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct QualityInspectionParameterBuilder {
    company_id: Option<Uuid>,
    template_id: Option<Uuid>,
    parameter_name: Option<String>,
    numeric: Option<bool>,
    min_value: Option<Decimal>,
    max_value: Option<Decimal>,
    spec_text: Option<String>,
}

impl QualityInspectionParameterBuilder {
    /// Set the company_id field (required)
    pub fn company_id(mut self, value: Uuid) -> Self {
        self.company_id = Some(value);
        self
    }

    /// Set the template_id field (required)
    pub fn template_id(mut self, value: Uuid) -> Self {
        self.template_id = Some(value);
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

    /// Set the spec_text field (optional)
    pub fn spec_text(mut self, value: String) -> Self {
        self.spec_text = Some(value);
        self
    }

    /// Build the QualityInspectionParameter entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<QualityInspectionParameter, String> {
        let company_id = self.company_id.ok_or_else(|| "company_id is required".to_string())?;
        let template_id = self.template_id.ok_or_else(|| "template_id is required".to_string())?;
        let parameter_name = self.parameter_name.ok_or_else(|| "parameter_name is required".to_string())?;

        Ok(QualityInspectionParameter {
            id: Uuid::new_v4(),
            company_id,
            template_id,
            parameter_name,
            numeric: self.numeric.unwrap_or(true),
            min_value: self.min_value,
            max_value: self.max_value,
            spec_text: self.spec_text,
            metadata: AuditMetadata::default(),
        })
    }
}
