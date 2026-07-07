use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::str::FromStr;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "inspection_type", rename_all = "snake_case")]
pub enum InspectionType {
    Incoming,
    InProcess,
    Outgoing,
}

impl std::fmt::Display for InspectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incoming => write!(f, "incoming"),
            Self::InProcess => write!(f, "in_process"),
            Self::Outgoing => write!(f, "outgoing"),
        }
    }
}

impl FromStr for InspectionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "incoming" => Ok(Self::Incoming),
            "in_process" => Ok(Self::InProcess),
            "outgoing" => Ok(Self::Outgoing),
            _ => Err(format!("Unknown InspectionType variant: {}", s)),
        }
    }
}

impl Default for InspectionType {
    fn default() -> Self {
        Self::Incoming
    }
}
