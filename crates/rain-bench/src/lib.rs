/// Number of concurrent tasks used in all benchmarks and tests.
pub const TASK_COUNT: usize = 50;

/// Number of store operations per task.
pub const OPS_PER_TASK: usize = 10;

/// Session ID prefix used to namespace each task's entries.
pub const SESSION_PREFIX: &str = "rain_bench_session";

/// Build the session ID for a given task index.
pub fn session_id(task_id: usize) -> String {
    format!("{SESSION_PREFIX}_{task_id}")
}

/// Build the store key for a given task and operation index.
pub fn entry_key(task_id: usize, op: usize) -> String {
    format!("rain_bench_task_{task_id}_op_{op}")
}

/// Build the entry content for a given task and operation index.
pub fn entry_content(task_id: usize, op: usize) -> String {
    format!("rain_bench_content task_{task_id} op_{op}")
}
