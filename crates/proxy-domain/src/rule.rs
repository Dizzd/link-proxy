use crate::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MappingRule {
    #[serde(default = "new_id")]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "yes")]
    pub enabled: bool,
    pub condition: Condition,
    pub action: Action,
    #[serde(default = "schema_version")]
    pub condition_schema_version: u32,
}
pub fn schema_version() -> u32 {
    1
}
