use crate::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mapping {
    #[serde(default = "new_id")]
    pub id: String,
    pub mapping_key: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub priority: i32,
    pub source_matcher: SourceMatcher,
    pub default_action: Action,
    #[serde(default)]
    pub rules: Vec<MappingRule>,
    #[serde(default = "now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "now")]
    pub updated_at: DateTime<Utc>,
}
