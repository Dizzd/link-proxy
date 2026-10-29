use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use proxy_cache::config::ImportLimits;
use proxy_import::{Format, parse};
use std::{hint::black_box, time::Duration};
fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("import_csv");
    group.sample_size(20);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(2));
    for count in [1_000, 10_000, 50_000] {
        let mut data =
            String::from("external_id,mapping_key,source_host,source_path,default_target_url\n");
        for i in 0..count {
            data.push_str(&format!(
                "id{i},key{i},example.com,/k{i},https://target.example/\n"
            ));
        }
        group.throughput(Throughput::Elements(count));
        group.bench_with_input(BenchmarkId::from_parameter(count), &data, |b, data| {
            b.iter(|| {
                black_box(
                    parse(
                        data.as_bytes(),
                        Format::Csv,
                        "bench.csv",
                        &ImportLimits::default(),
                        &[],
                    )
                    .unwrap(),
                )
            })
        });
    }
    group.finish();
}
criterion_group!(benches, bench);
criterion_main!(benches);
