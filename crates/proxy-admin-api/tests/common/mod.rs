#![allow(dead_code)]
use proxy_application::ControlPlane;
use proxy_cache::config::Settings;
use proxy_domain::*;
use std::sync::Arc;
pub fn mapping(key: &str, path: &str) -> Mapping {
    serde_json::from_value(serde_json::json!({"id":format!("id-{key}"),"mapping_key":key,"source_matcher":{"hosts":["example.com"],"path_type":"exact","path_value":path},"default_action":{"type":"redirect","status":302,"target_url":"https://destination.example/"}})).unwrap()
}
pub fn configuration(mappings: Vec<Mapping>) -> Configuration {
    Configuration {
        schema_version: 1,
        mappings,
        ..Default::default()
    }
}
#[allow(dead_code)]
pub async fn control() -> Arc<ControlPlane> {
    ControlPlane::open(Settings {
        database_url: "sqlite::memory:".into(),
        ..Default::default()
    })
    .await
    .unwrap()
}
