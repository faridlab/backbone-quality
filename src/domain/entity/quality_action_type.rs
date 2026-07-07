use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::str::FromStr;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "quality_action_type", rename_all = "snake_case")]
pub enum QualityActionType {
    Corrective,
    Preventive,
}

impl std::fmt::Display for QualityActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Corrective => write!(f, "corrective"),
            Self::Preventive => write!(f, "preventive"),
        }
    }
}

impl FromStr for QualityActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "corrective" => Ok(Self::Corrective),
            "preventive" => Ok(Self::Preventive),
            _ => Err(format!("Unknown QualityActionType variant: {}", s)),
        }
    }
}

impl Default for QualityActionType {
    fn default() -> Self {
        Self::Corrective
    }
}
