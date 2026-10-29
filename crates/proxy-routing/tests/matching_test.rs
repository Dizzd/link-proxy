use proxy_domain::{Configuration, Mapping};
use proxy_routing::RoutingSnapshot;
#[test]
fn method_mismatch_has_path_without_selecting_action() {
    let m:Mapping=serde_json::from_value(serde_json::json!({"mapping_key":"a","source_matcher":{"hosts":["example.com"],"path_type":"exact","path_value":"/a","methods":["GET"]},"default_action":{"type":"redirect","status":302,"target_url":"https://target.example/"}})).unwrap();
    let snapshot = RoutingSnapshot::compile(
        1,
        Configuration {
            schema_version: 1,
            mappings: vec![m],
            ..Default::default()
        },
        &[],
    )
    .unwrap();
    assert!(snapshot.lookup("example.com", "/a", "POST").is_none());
    assert!(snapshot.path_exists("example.com", "/a"));
    assert!(!snapshot.path_exists("example.com", "/unknown"));
}
