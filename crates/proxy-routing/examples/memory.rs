use proxy_domain::{Configuration, Mapping};
use proxy_routing::RoutingSnapshot;
fn main() {
    for count in [1_000, 10_000, 100_000] {
        let mappings=(0..count).map(|i|serde_json::from_value::<Mapping>(serde_json::json!({"id":format!("m{i}"),"mapping_key":format!("k{i}"),"source_matcher":{"hosts":["example.com"],"path_type":"exact","path_value":format!("/k{i}")},"default_action":{"type":"redirect","status":302,"target_url":"https://target.example/"}})).unwrap()).collect();
        let start = std::time::Instant::now();
        let snapshot = RoutingSnapshot::compile(
            1,
            Configuration {
                schema_version: 1,
                mappings,
                ..Default::default()
            },
            &[],
        )
        .unwrap();
        let rss = std::fs::read_to_string("/proc/self/status").unwrap();
        let rss = rss.lines().find(|l| l.starts_with("VmRSS:")).unwrap();
        println!(
            "mappings={count} compile_ms={} process_{rss}",
            start.elapsed().as_millis()
        );
        std::hint::black_box(snapshot);
    }
}
