# rain-bench

Concurrency benchmarks for the rain-labs memory subsystem.

## What's Benchmarked

- `store_concurrent` — concurrent write throughput across memory backends
- `recall_integrity` — concurrent read correctness under write pressure

## Run

```bash
cargo bench -p rain-bench
```

## Design

Benches use [criterion](https://bheisler.github.io/criterion.rs/) with the `async_tokio` feature for true async execution. Each benchmark:
1. Spawns N concurrent tasks hitting the same memory backend
2. Measures throughput (ops/sec) and latency percentiles
3. Verifies recall correctness after the burst

See `benches/` for individual benchmark definitions.
