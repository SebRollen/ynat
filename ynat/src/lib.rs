mod app;
pub mod app_core;
mod background;
mod cache;
pub mod commands;
pub mod events;
pub mod input;
pub mod log_buffer;
pub mod logging;
pub mod state;
pub mod ui;
mod utils;

pub use app::App;

// Always expose testing module (integration tests need it)
pub mod testing;
