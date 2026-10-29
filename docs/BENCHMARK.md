# Benchmarks

## Reproduce
```bash
cargo bench -p proxy-routing --bench routing_bench
cargo bench -p proxy-import --bench import_bench
cargo run --release -p proxy-routing --example memory
cargo test --test imports import_large_10000_and_50000_records -- --ignored --nocapture
```

Criterion benches live in owning crates. Routing covers indexed exact lookup and full snapshot compilation for 1,000/10,000/100,000 mappings, plus nested device/time conditions. Compilation setup clones the input outside the measured routine using iter_batched; measured work includes compiler/index construction and output destruction. Lookup includes Host normalization. CSV throughput covers parsing and per-record host/path/action/condition validation, excluding SQL commit and publication. Samples default to 20, one-second warmup and at least two-second measurement; 100k compilation automatically extends sample collection.

The memory example reports actual Linux process VmRSS after snapshot construction. It includes allocator retention, input/config copies and runtime process overhead, not a per-record allocation proof. Cases execute sequentially, so later RSS may include retained pages from prior cases. Compare fresh-process baselines for production sizing.

The current CSV parser includes per-record semantic validation. Criterion compares this run with an earlier decode-only baseline; its change report therefore includes the cost of the added validation work.

## Measured results, 2026-10-29

Host: Intel Core i5-11320H, 8 logical CPUs, x86_64 Linux WSL2 6.6.87.2; Rust 1.98.1, optimized release profile, Criterion 20 samples. Raw output and host details are in verification/.

| Mappings | Exact lookup central estimate | Snapshot compilation central estimate | Process RSS after compilation |
|---:|---:|---:|---:|
| 1,000 | 191.13 ns | 1.8000 ms | 4,188 kB |
| 10,000 | 193.54 ns | 28.974 ms | 17,556 kB |
| 100,000 | 168.71 ns | 322.42 ms | 148,880 kB |

Nested device/time rule evaluation: 277.47 ns central estimate (273.29-280.72 ns interval). Lookup at 100k: 164.16-173.85 ns interval. Compilation at 100k: 319.25-325.96 ms interval. These are microbenchmark estimates, not p95/p99 HTTP latencies. Differences between sizes include normal host frequency/cache variation.

| CSV records | Parse central estimate | Parse throughput |
|---:|---:|---:|
| 1,000 | 1.4062 ms | 0.71116 million records/s |
| 10,000 | 17.272 ms | 0.57896 million records/s |
| 50,000 | 66.575 ms | 0.75104 million records/s |

Separate functional scale test, debug build and SQLite **in-memory** database: 10,000 mappings serialized, parsed, staged, validated and activated in 4.645334636 s; 50,000 in 22.385757377 s. Input Mapping construction is excluded, serialization is included. These timings do not measure disk durability or HTTP upload overhead. Persistent SQLite restart/durability is covered by separate integration/process tests.

No HTTP QPS, tail latency under load or container memory SLA is claimed. Production benchmarking should control CPU affinity, concurrency, TLS, upstream latency, network and logging, with disk-backed SQLite for control-plane batch measurements.
