use crate::commands::handlers;
use crate::events::{AppCommand, DataEvent};
use crate::input::KeyEvent;
use crate::state::{reducer, AppState};

/// Trait for handling command execution (production = real loader + tasks, test = mock)
///
/// This trait abstracts the side effects of command execution, allowing tests
/// to inject a mock implementation that doesn't spawn background tasks or make API calls.
pub trait DataEventHandler {
    /// Execute a command with access to mutable state
    ///
    /// In production, this spawns background tasks and manages the data loader.
    /// In tests, this can update state synchronously without side effects.
    fn execute_with_context(&mut self, command: AppCommand, state: &mut AppState);
}

/// Testable application core without terminal dependencies
///
/// Generic over H (handler) for zero-cost abstraction. The handler type determines
/// how commands are executed - in production it spawns tasks, in tests it updates
/// state synchronously.
pub struct AppCore<H: DataEventHandler> {
    ui_state: AppState,
    handler: H,
}

impl<H: DataEventHandler> AppCore<H> {
    /// Create a new application core with the given handler
    pub fn new(handler: H) -> Self {
        Self {
            ui_state: AppState::new(),
            handler,
        }
    }

    /// Handle keyboard input and execute the resulting command
    ///
    /// This is the main entry point for user input. It:
    /// 1. Translates the key press to an AppCommand via the handler
    /// 2. Executes the command using the configured handler
    pub fn handle_key(&mut self, event: KeyEvent) {
        if let Some(command) = handlers::handle_key_input(event, &self.ui_state) {
            self.handler
                .execute_with_context(command, &mut self.ui_state);
        }
    }

    /// Handle a data event (for test injection or async results)
    ///
    /// Data events come from background tasks in production (API responses, cache loads).
    /// In tests, you can inject events directly to simulate async operations.
    pub fn handle_data_event(&mut self, event: DataEvent) {
        reducer::reduce_data_event(&mut self.ui_state, event);
    }

    /// Get read-only access to the current UI state (for rendering or assertions)
    pub fn state(&self) -> &AppState {
        &self.ui_state
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.ui_state.should_quit
    }
}
