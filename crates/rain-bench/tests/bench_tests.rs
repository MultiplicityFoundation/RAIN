/// Integration tests for rain-bench Phase 1.
///
/// Tests 1-10 from the eng review test plan.
use rain_bench::{entry_content, entry_key, session_id, OPS_PER_TASK, TASK_COUNT};
use rain_labs::memory::{Memory, MemoryCategory, SqliteMemory};
use std::sync::Arc;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test 1: 50 concurrent tasks storing — no panics, no data loss
// ---------------------------------------------------------------------------
#[tokio::test]
async fn rain_bench_store_concurrent_50_tasks() {
    let tmp = TempDir::new().unwrap();
    let memory: Arc<dyn Memory> = Arc::new(SqliteMemory::new(tmp.path()).unwrap());

    let mut handles = tokio::task::JoinSet::new();
    for task_id in 0..TASK_COUNT {
        let m = memory.clone();
        handles.spawn(async move {
            for op in 0..OPS_PER_TASK {
                m.store(
                    &entry_key(task_id, op),
                    &entry_content(task_id, op),
                    MemoryCategory::Conversation,
                    Some(&session_id(task_id)),
                )
                .await
                .expect("store should succeed");
            }
        });
    }
    while let Some(r) = handles.join_next().await {
        r.expect("task should not panic");
    }
}

// ---------------------------------------------------------------------------
// Test 2: recall after concurrent store — stored count == recalled count per task
// ---------------------------------------------------------------------------
#[tokio::test]
async fn rain_bench_recall_after_concurrent_store() {
    let tmp = TempDir::new().unwrap();
    let memory: Arc<dyn Memory> = Arc::new(SqliteMemory::new(tmp.path()).unwrap());

    let mut handles = tokio::task::JoinSet::new();
    for task_id in 0..TASK_COUNT {
        let m = memory.clone();
        handles.spawn(async move {
            for op in 0..OPS_PER_TASK {
                m.store(
                    &entry_key(task_id, op),
                    &entry_content(task_id, op),
                    MemoryCategory::Conversation,
                    Some(&session_id(task_id)),
                )
                .await
                .unwrap();
            }
        });
    }
    while let Some(r) = handles.join_next().await {
        r.unwrap();
    }

    for task_id in 0..TASK_COUNT {
        let sid = session_id(task_id);
        let entries = memory
            .recall(
                &format!("task_{task_id}"),
                OPS_PER_TASK + 1,
                Some(&sid),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(
            entries.len(),
            OPS_PER_TASK,
            "task {task_id}: expected {OPS_PER_TASK} entries, got {}",
            entries.len()
        );
    }
}

// ---------------------------------------------------------------------------
// Test 3: integrity_failures == 0 on a clean DB
// ---------------------------------------------------------------------------
#[tokio::test]
async fn rain_bench_integrity_failures_zero() {
    let tmp = TempDir::new().unwrap();
    let memory: Arc<dyn Memory> = Arc::new(SqliteMemory::new(tmp.path()).unwrap());

    let mut handles = tokio::task::JoinSet::new();
    for task_id in 0..TASK_COUNT {
        let m = memory.clone();
        handles.spawn(async move {
            for op in 0..OPS_PER_TASK {
                m.store(
                    &entry_key(task_id, op),
                    &entry_content(task_id, op),
                    MemoryCategory::Conversation,
                    Some(&session_id(task_id)),
                )
                .await
                .unwrap();
            }
        });
    }
    while let Some(r) = handles.join_next().await {
        r.unwrap();
    }

    let mut failures = 0usize;
    for task_id in 0..TASK_COUNT {
        let sid = session_id(task_id);
        let entries = memory
            .recall(
                &format!("task_{task_id}"),
                OPS_PER_TASK + 1,
                Some(&sid),
                None,
                None,
            )
            .await
            .unwrap();
        if entries.len() < OPS_PER_TASK {
            failures += 1;
        }
    }
    assert_eq!(failures, 0, "expected 0 integrity failures, got {failures}");
}

// ---------------------------------------------------------------------------
// Test 4: TempDir cleaned up after drop (DB file auto-deleted)
// ---------------------------------------------------------------------------
#[test]
fn rain_bench_tempfile_cleanup() {
    let db_path;
    {
        let tmp = TempDir::new().unwrap();
        // SqliteMemory creates the DB at <workspace>/memory/brain.db
        db_path = tmp.path().join("memory").join("brain.db");
        let _mem = SqliteMemory::new(tmp.path()).unwrap();
        assert!(db_path.exists(), "memory/brain.db should exist while TempDir is alive");
        // TempDir drops here
    }
    assert!(!db_path.exists(), "memory/brain.db should be deleted after TempDir is dropped");
}

// ---------------------------------------------------------------------------
// Test 5: Arc<dyn Memory> is Send + Sync (compile-time assertion)
// ---------------------------------------------------------------------------
#[test]
fn rain_bench_arc_dyn_memory_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Arc<dyn Memory>>();
}

// ---------------------------------------------------------------------------
// Test 6: single task store + recall — baseline correctness
// ---------------------------------------------------------------------------
#[tokio::test]
async fn rain_bench_single_task_store_recall() {
    let tmp = TempDir::new().unwrap();
    let memory: Arc<dyn Memory> = Arc::new(SqliteMemory::new(tmp.path()).unwrap());

    for op in 0..OPS_PER_TASK {
        memory
            .store(
                &entry_key(0, op),
                &entry_content(0, op),
                MemoryCategory::Conversation,
                Some(&session_id(0)),
            )
            .await
            .unwrap();
    }

    let entries = memory
        .recall("task_0", OPS_PER_TASK + 1, Some(&session_id(0)), None, None)
        .await
        .unwrap();
    assert_eq!(entries.len(), OPS_PER_TASK);
}

// ---------------------------------------------------------------------------
// Test 7: high concurrency — 200 tasks, no deadlock (5s timeout guard)
// ---------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn rain_bench_high_concurrency_no_deadlock() {
    const HIGH_TASK_COUNT: usize = 200;
    let tmp = TempDir::new().unwrap();
    let memory: Arc<dyn Memory> = Arc::new(SqliteMemory::new(tmp.path()).unwrap());

    let test = async {
        let mut handles = tokio::task::JoinSet::new();
        for task_id in 0..HIGH_TASK_COUNT {
            let m = memory.clone();
            handles.spawn(async move {
                m.store(
                    &format!("high_task_{task_id}"),
                    &format!("content_{task_id}"),
                    MemoryCategory::Conversation,
                    Some(&format!("session_{task_id}")),
                )
                .await
                .unwrap();
            });
        }
        while let Some(r) = handles.join_next().await {
            r.unwrap();
        }
    };

    tokio::time::timeout(std::time::Duration::from_secs(30), test)
        .await
        .expect("200 concurrent tasks should complete within 30 seconds — possible deadlock");
}

// ---------------------------------------------------------------------------
// Test 8: recall on empty DB returns empty vec, no panic
// ---------------------------------------------------------------------------
#[tokio::test]
async fn rain_bench_empty_recall_returns_empty() {
    let tmp = TempDir::new().unwrap();
    let memory: Arc<dyn Memory> = Arc::new(SqliteMemory::new(tmp.path()).unwrap());

    let entries = memory
        .recall("anything", 10, Some("empty_session"), None, None)
        .await
        .unwrap();
    assert!(entries.is_empty(), "recall on empty DB should return empty vec");
}

// ---------------------------------------------------------------------------
// Test 9: session isolation — entries from session A not returned for session B
// ---------------------------------------------------------------------------
#[tokio::test]
async fn rain_bench_session_isolation() {
    let tmp = TempDir::new().unwrap();
    let memory: Arc<dyn Memory> = Arc::new(SqliteMemory::new(tmp.path()).unwrap());

    // Store an entry in session A.
    memory
        .store(
            "isolation_key",
            "isolation content",
            MemoryCategory::Core,
            Some("session_a"),
        )
        .await
        .unwrap();

    // Recall from session B — should not see session A's entry.
    let entries = memory
        .recall("isolation", 10, Some("session_b"), None, None)
        .await
        .unwrap();
    assert!(
        entries.is_empty(),
        "session_b should not see entries stored under session_a"
    );

    // Sanity: session A can see its own entry.
    let entries_a = memory
        .recall("isolation", 10, Some("session_a"), None, None)
        .await
        .unwrap();
    assert_eq!(entries_a.len(), 1, "session_a should see its own entry");
}

// ---------------------------------------------------------------------------
// Test 10: SqliteMemory health check passes on a fresh DB
// ---------------------------------------------------------------------------
#[tokio::test]
async fn rain_bench_sqlite_memory_health_check_passes() {
    let tmp = TempDir::new().unwrap();
    let memory = SqliteMemory::new(tmp.path()).unwrap();
    assert!(
        memory.health_check().await,
        "SqliteMemory health_check should return true on a fresh DB"
    );
}
