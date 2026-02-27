use crossterm::event::{KeyCode, KeyEvent as CrosstermKeyEvent, KeyModifiers};

/// Framework-agnostic key representation for testability
///
/// This enum abstracts away the crossterm-specific KeyCode type,
/// allowing tests to inject keyboard input without depending on crossterm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Tab,
    BackTab,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
}

/// Modifier key state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

/// Key event with modifier state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl KeyEvent {
    /// Create a new KeyEvent with the given key and no modifiers
    pub fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::default(),
        }
    }

    /// Create a new KeyEvent with the given key and Ctrl modifier
    pub fn with_ctrl(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers {
                ctrl: true,
                ..Default::default()
            },
        }
    }
}

impl From<KeyCode> for Key {
    fn from(code: KeyCode) -> Self {
        match code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::BackTab,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            // For any unmapped keys, treat as null char
            _ => Key::Char('\0'),
        }
    }
}

impl From<CrosstermKeyEvent> for KeyEvent {
    fn from(event: CrosstermKeyEvent) -> Self {
        Self {
            key: Key::from(event.code),
            modifiers: Modifiers {
                ctrl: event.modifiers.contains(KeyModifiers::CONTROL),
                alt: event.modifiers.contains(KeyModifiers::ALT),
                shift: event.modifiers.contains(KeyModifiers::SHIFT),
            },
        }
    }
}
