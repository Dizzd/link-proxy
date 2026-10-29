use crate::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default = "schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub domains: Vec<Domain>,
    #[serde(default)]
    pub upstreams: Vec<Upstream>,
    #[serde(default)]
    pub mappings: Vec<Mapping>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigRevision {
    pub version: i64,
    pub status: String,
    pub checksum: String,
    pub created_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub actor: String,
    pub resource_id: Option<String>,
    pub timestamp: String,
    pub id: i64,
    pub operation: String,
    pub revision: i64,
    pub created_at: String,
}
