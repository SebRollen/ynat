pub mod data_loader;

use std::collections::HashMap;
use std::future::Future;
use tokio::task::JoinHandle;

/// Manages background tasks for data loading
/// Tracks running tasks and provides cancellation support
pub struct BackgroundTaskManager {
    tasks: HashMap<String, JoinHandle<()>>,
}

impl BackgroundTaskManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    /// Spawn a background data loading task
    /// If a task with the same ID already exists, it will be cancelled first
    pub fn spawn_load_task<F>(&mut self, task_id: String, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // Cancel existing task with same ID (prevents stale data)
        if let Some(handle) = self.tasks.remove(&task_id) {
            handle.abort();
        }

        // Spawn new task
        let handle = tokio::spawn(future);
        self.tasks.insert(task_id, handle);
    }

    /// Cancel all running tasks (used on shutdown)
    pub fn cancel_all(&mut self) {
        for (_, handle) in self.tasks.drain() {
            handle.abort();
        }
    }
}

impl Default for BackgroundTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BackgroundTaskManager {
    fn drop(&mut self) {
        self.cancel_all();
    }
}
