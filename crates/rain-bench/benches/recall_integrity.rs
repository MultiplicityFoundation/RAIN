/// Benchmark: data durability under concurrent write contention.
///
/// 50 concurrent tasks each store `OPS_PER_TASK` entries. After all tasks complete,
/// `recall()` is called for each task's session. The benchmark asserts that every
/// entry stored is retrievable — testing FTS5 index consistency and mutex fairness,
/// not trivially satisfied by ACID write durability alone.
///
/// `integrity_failures` = task slots where recalled count < stored count.
/// The benchmark panics if any failures are detected.
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rain_bench::{entry_content, entry_key, session_id, OPS_PER_TASK, TASK_COUNT};
use rain_labs::memory::{Memory, MemoryCategory, SqliteMemory};
use std::sync::Arc;
use tempfile::TempDir;

fn bench_recall_integrity(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    c.bench_function("recall_integrity_50_tasks", |b| {
        b.to_async(&rt).iter_batched(
            // Setup: fresh SqliteMemory per iteration so each run starts from a clean state.
            || {
                let tmp = TempDir::new().expect("TempDir::new failed");
                let mem: Arc<dyn Memory> =
                    Arc::new(SqliteMemory::new(tmp.path()).expect("SqliteMemory::new failed"));
                (tmp, mem)
            },
            // Routine: concurrent stores then recall-based integrity check.
            |(_tmp, mem)| async move {
                // Phase 1: 50 concurrent tasks, each storing OPS_PER_TASK entries.
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

                // Phase 2: recall each task's entries and verify count.
                let mut integrity_failures = 0usize;
                for task_id in 0..TASK_COUNT {
                    let sid = session_id(task_id);
                    // Query by a unique prefix that matches only this task's content.
                    let query = format!("task_{task_id}");
                    let entries = mem
                        .recall(&query, OPS_PER_TASK + 1, Some(&sid), None, None)
                        .await
                        .expect("recall failed");
                    if entries.len() < OPS_PER_TASK {
                        integrity_failures += 1;
                    }
                }

                assert_eq!(
                    integrity_failures, 0,
                    "{integrity_failures} task(s) had fewer recalled entries than stored entries"
                );
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_recall_integrity);
criterion_main!(benches);
