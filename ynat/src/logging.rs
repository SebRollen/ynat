use anyhow::Result;
use chrono::Local;
use std::path::PathBuf;
use tracing::Subscriber;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, registry::LookupSpan, util::SubscriberInitExt, EnvFilter, Layer,
};

use crate::log_buffer::{LogBuffer, LogEntry};

/// Initialize tracing with file-based logging
/// Logs are written to ~/.config/ynat/logs/ynat-YYYY-MM-DD-HH-MM-SS.log
pub fn init_logging() -> Result<PathBuf> {
    // Get config directory
    let config_dir = dirs::config_dir()
        .ok_or(anyhow::anyhow!("Could not find config directory"))?
        .join("ynat");

    // Create logs directory
    let logs_dir = config_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;

    // Create timestamped log file name
    let timestamp = Local::now().format("%Y-%m-%d-%H-%M-%S");
    let log_filename = format!("ynat-{}.log", timestamp);
    let log_path = logs_dir.join(&log_filename);

    // Create file appender (non-blocking for better performance)
    let file_appender = tracing_appender::rolling::never(&logs_dir, &log_filename);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Set up formatting layer for file output
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false) // No ANSI codes in log file
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    // Set up filter (default to INFO, can be overridden with RUST_LOG env var)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Initialize subscriber
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .init();

    // Keep the guard alive for the lifetime of the program
    // We'll leak it so it doesn't get dropped
    std::mem::forget(_guard);

    Ok(log_path)
}

/// Initialize tracing with file-based logging and an in-memory buffer for UI display
pub fn init_logging_with_buffer(buffer: LogBuffer) -> Result<PathBuf> {
    // Get config directory
    let config_dir = dirs::config_dir()
        .ok_or(anyhow::anyhow!("Could not find config directory"))?
        .join("ynat");

    // Create logs directory
    let logs_dir = config_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;

    // Create timestamped log file name
    let timestamp = Local::now().format("%Y-%m-%d-%H-%M-%S");
    let log_filename = format!("ynat-{}.log", timestamp);
    let log_path = logs_dir.join(&log_filename);

    // Create file appender (non-blocking for better performance)
    let file_appender = tracing_appender::rolling::never(&logs_dir, &log_filename);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Set up formatting layer for file output
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    // Set up filter (default to INFO, can be overridden with RUST_LOG env var)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Create buffer layer for UI display
    let buffer_layer = LogBufferLayer::new(buffer);

    // Initialize subscriber with both layers
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(buffer_layer)
        .init();

    // Keep the guard alive for the lifetime of the program
    std::mem::forget(_guard);

    Ok(log_path)
}

/// A tracing layer that captures log entries to an in-memory buffer
pub struct LogBufferLayer {
    buffer: LogBuffer,
}

impl LogBufferLayer {
    pub fn new(buffer: LogBuffer) -> Self {
        Self { buffer }
    }
}

impl<S> Layer<S> for LogBufferLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Extract message from event
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let entry = LogEntry {
            timestamp: chrono::Local::now(),
            level: *event.metadata().level(),
            target: event.metadata().target().to_string(),
            message: visitor.message,
        };

        self.buffer.push(entry);
    }
}

/// Visitor to extract the message field from a tracing event
#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
}
