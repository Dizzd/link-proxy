use crate::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Upstream {
    #[serde(default = "new_id")]
    pub id: String,
    pub name: String,
    pub scheme: String,
    pub hostname: String,
    pub port: u16,
    #[serde(default = "yes")]
    pub tls_verify: bool,
    #[serde(default = "connect_ms")]
    pub connect_timeout_ms: u64,
    #[serde(default = "read_ms")]
    pub read_timeout_ms: u64,
}
fn connect_ms() -> u64 {
    3000
}
fn read_ms() -> u64 {
    30000
}
