use crate::app_core::{AppCore, DataEventHandler};
use crate::commands::executor;
use crate::events::{AppCommand, DataEvent};
use crate::input::{Key, KeyEvent};
use crate::state::AppState;
use crate::ui::screens::Screen;

/// Mock data event handler for tests (no real async tasks)
///
/// This handler executes commands synchronously using execute_command_sync,
/// which updates state without spawning background tasks or making API calls.
pub struct MockDataHandler;

impl MockDataHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockDataHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DataEventHandler for MockDataHandler {
    fn execute_with_context(&mut self, command: AppCommand, state: &mut AppState) {
        // Execute command synchronously without spawning tasks
        executor::execute_command_sync(command, state);
    }
}

pub struct TestApp {
    core: AppCore<MockDataHandler>,
}

impl TestApp {
    /// Create a new test app with mock handler
    pub fn new() -> Self {
        Self {
            core: AppCore::new(MockDataHandler::new()),
        }
    }

    /// Send a single key event
    pub fn send_key(&mut self, key: Key) {
        self.core.handle_key(KeyEvent::new(key));
    }

    /// Send a key event with modifiers
    pub fn send_key_event(&mut self, event: KeyEvent) {
        self.core.handle_key(event);
    }

    /// Send multiple keys in sequence
    pub fn send_keys(&mut self, keys: &[Key]) {
        for key in keys {
            self.send_key(*key);
        }
    }

    /// Inject a data event (simulate API response or cache load)
    pub fn send_data_event(&mut self, event: DataEvent) {
        self.core.handle_data_event(event);
    }

    /// Get read-only access to current state
    pub fn state(&self) -> &AppState {
        self.core.state()
    }

    /// Assert that the app is on a specific screen type
    ///
    /// Uses discriminant comparison to check screen type without
    /// requiring full equality of state.
    pub fn assert_screen_type(&self, expected_discriminant: std::mem::Discriminant<Screen>) {
        let current = self.state().current_screen();
        assert_eq!(
            std::mem::discriminant(current),
            expected_discriminant,
            "Expected different screen. Current: {:?}",
            current
        );
    }

    /// Assert that the app should quit
    pub fn assert_should_quit(&self) {
        assert!(
            self.core.should_quit(),
            "App should be marked for quit but is not"
        );
    }

    /// Assert that the app should NOT quit
    pub fn assert_not_quit(&self) {
        assert!(
            !self.core.should_quit(),
            "App should NOT be marked for quit but is"
        );
    }
}

impl Default for TestApp {
    fn default() -> Self {
        Self::new()
    }
}
