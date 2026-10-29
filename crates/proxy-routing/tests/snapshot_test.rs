use proxy_domain::{Action, Configuration, Mapping};
use proxy_routing::RoutingSnapshot;
#[test]
fn old_snapshot_is_immutable_when_a_new_configuration_is_compiled() {
    let mapping:Mapping=serde_json::from_value(serde_json::json!({"mapping_key":"stable","source_matcher":{"hosts":["example.com"],"path_type":"exact","path_value":"/stable"},"default_action":{"type":"redirect","status":302,"target_url":"https://old.example/"}})).unwrap();
    let mut config = Configuration {
        schema_version: 1,
        mappings: vec![mapping],
        ..Default::default()
    };
    let old = RoutingSnapshot::compile(1, config.clone(), &[]).unwrap();
    if let Action::Redirect { target_url, .. } = &mut config.mappings[0].default_action {
        *target_url = "https://new.example/".into();
    }
    let new = RoutingSnapshot::compile(2, config, &[]).unwrap();
    assert!(
        matches!(&old.lookup("example.com","/stable","GET").unwrap().mapping.mapping.default_action,Action::Redirect{target_url,..} if target_url=="https://old.example/")
    );
    assert_eq!(new.revision, 2);
}
