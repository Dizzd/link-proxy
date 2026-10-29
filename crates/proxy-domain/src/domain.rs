use crate::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Domain {
    #[serde(default = "new_id")]
    pub id: String,
    pub hostname: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default = "now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "now")]
    pub updated_at: DateTime<Utc>,
}
