use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use tracing::Level;

/// A single log entry captured from tracing
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub level: Level,
    pub target: String,
    pub message: String,
}

/// Thread-safe circular buffer for log entries
#[derive(Debug, Clone)]
pub struct LogBuffer {
    entries: Arc<RwLock<VecDeque<LogEntry>>>,
    max_entries: usize,
}

impl LogBuffer {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    pub fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.write().unwrap();
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries.read().unwrap().iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }
}
