use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use proxy_domain::{Configuration, Mapping};
use proxy_routing::{
    RoutingSnapshot,
    conditions::{CompiledCondition, RequestContext},
};
use std::{hint::black_box, time::Duration};
fn config(count: usize) -> Configuration {
    Configuration {schema_version:1,mappings:(0..count).map(|i|serde_json::from_value::<Mapping>(serde_json::json!({"id":format!("m{i}"),"mapping_key":format!("k{i}"),"source_matcher":{"hosts":["example.com"],"path_type":"exact","path_value":format!("/k{i}")},"default_action":{"type":"redirect","status":302,"target_url":"https://destination.example/"}})).unwrap()).collect(),..Default::default()}
}
fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing");
    group.sample_size(20);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(2));
    for count in [1_000, 10_000, 100_000] {
        let cfg = config(count);
        let snapshot = RoutingSnapshot::compile(1, cfg.clone(), &[]).unwrap();
        let path = format!("/k{}", count - 1);
        group.bench_with_input(BenchmarkId::new("exact_lookup", count), &count, |b, _| {
            b.iter(|| black_box(snapshot.lookup(black_box("example.com"), black_box(&path), "GET")))
        });
        group.bench_with_input(
            BenchmarkId::new("snapshot_compile", count),
            &count,
            |b, _| {
                b.iter_batched(
                    || cfg.clone(),
                    |input| black_box(RoutingSnapshot::compile(1, input, &[]).unwrap()),
                    criterion::BatchSize::LargeInput,
                )
            },
        );
    }
    let condition=CompiledCondition::compile(&serde_json::from_value(serde_json::json!({"type":"all","conditions":[{"type":"device_type","in":["mobile"]},{"type":"time_window","timezone":"Asia/Ho_Chi_Minh","days_of_week":["mon","tue","wed","thu","fri"],"start":"08:00","end":"18:00"}]})).unwrap()).unwrap();
    let ctx = RequestContext {
        now: "2026-10-29T01:00:00Z".parse().unwrap(),
        user_agent: "iPhone Mobile",
        query: "",
    };
    group.bench_function("condition_evaluation", |b| {
        b.iter(|| black_box(condition.matches(black_box(&ctx))))
    });
    group.finish();
}
criterion_group!(benches, bench);
criterion_main!(benches);
