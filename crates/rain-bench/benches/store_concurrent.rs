/// Benchmark: throughput of 50 concurrent `Memory::store()` calls on a shared `SqliteMemory`.
///
/// One `SqliteMemory` instance is constructed once per benchmark group (not per iteration),
/// wrapping a tempdir on disk. All 50 tasks share it via `Arc`. This measures real
/// `parking_lot::Mutex` contention — the same contention profile as production.
use criterion::{criterion_group, criterion_main, Criterion};
use rain_bench::{entry_content, entry_key, session_id, OPS_PER_TASK, TASK_COUNT};
use rain_labs::memory::{Memory, MemoryCategory, SqliteMemory};
use std::sync::Arc;
use tempfile::TempDir;

fn bench_store_concurrent(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    // Setup outside the measured loop: one SqliteMemory for the whole bench group.
    let tmp = TempDir::new().unwrap();
    let memory: Arc<dyn Memory> =
        Arc::new(SqliteMemory::new(tmp.path()).expect("SqliteMemory::new failed"));

    c.bench_function("store_50_concurrent_tasks", |b| {
        b.to_async(&rt).iter(|| {
            let mem = memory.clone();
            async move {
                let mut handles = tokio::task::JoinSet::new();
                for task_id in 0..TASK_COUNT {
                    let m = mem.clone();
                    handles.spawn(async move {
                        for op in 0..OPS_PER_TASK {
                            m.store(
                                &entry_key(task_id, op),
                                &entry_content(task_id, op),
                                MemoryCategory::Conversation,
                                Some(&session_id(task_id)),
                            )
                            .await
                            .expect("store failed");
                        }
                    });
                }
                while let Some(result) = handles.join_next().await {
                    result.expect("task panicked");
                }
            }
        });
    });

    // tmp is kept alive until here so the DB file persists for the full bench run.
    drop(tmp);
}

criterion_group!(benches, bench_store_concurrent);
criterion_main!(benches);
