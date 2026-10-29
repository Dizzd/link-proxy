use proxy_cache::config::ImportLimits;
use proxy_domain::Configuration;
use proxy_import::{Format, export, parse};
#[test]
fn xlsx_preserves_dependencies_and_negative_rule_priority() {
    let c:Configuration=serde_json::from_value(serde_json::json!({"schema_version":1,"domains":[{"id":"d","hostname":"example.com"}],"upstreams":[{"id":"u","name":"backend","scheme":"http","hostname":"backend.example","port":80}],"mappings":[{"id":"m","mapping_key":"m","source_matcher":{"hosts":["example.com"],"path_type":"exact","path_value":"/a"},"default_action":{"type":"reverse_proxy","upstream_id":"u"},"rules":[{"id":"r","priority":-10,"condition":{"type":"device_type","in":["mobile"]},"action":{"type":"redirect","status":302,"target_url":"https://mobile.example/"}}]}]})).unwrap();
    let bytes = export(&c, Format::Xlsx).unwrap();
    let (parsed, errors) = parse(
        &bytes,
        Format::Xlsx,
        "test.xlsx",
        &ImportLimits::default(),
        &[],
    )
    .unwrap();
    assert!(errors.is_empty());
    assert_eq!(
        serde_json::to_value(parsed.upstreams).unwrap(),
        serde_json::to_value(c.upstreams).unwrap()
    );
    assert_eq!(
        serde_json::to_value(parsed.domains).unwrap(),
        serde_json::to_value(c.domains).unwrap()
    );
    assert_eq!(parsed.mappings[0].rules[0].priority, -10);
}

#[test]
fn semantic_errors_identify_each_invalid_import_record() {
    let data=b"external_id,mapping_key,source_host,source_path,default_target_url\na,a,bad host,/a,https://safe.example/\nb,b,example.com,/b,file:///etc/passwd\n";
    let (_, errors) = parse(
        data,
        Format::Csv,
        "invalid.csv",
        &ImportLimits::default(),
        &[],
    )
    .unwrap();
    assert_eq!(errors.len(), 2);
    assert!(errors[0].location.contains("external_id=a"));
    assert!(errors[1].location.contains("external_id=b"));
}
