//! Centralized theme constants and style functions for consistent UI styling.
//!
//! All colors, layout constants, and common styles should be defined here
//! to ensure visual consistency across all screens and components.

use ratatui::style::{Color, Modifier, Style};

// =============================================================================
// Colors
// =============================================================================

/// Color for positive amounts (inflows, gains)
pub const COLOR_POSITIVE: Color = Color::Green;

/// Color for negative amounts (outflows, expenses)
pub const COLOR_NEGATIVE: Color = Color::Red;

/// Color for zero amounts
pub const COLOR_ZERO: Color = Color::DarkGray;

/// Background color for selected/highlighted rows
pub const COLOR_SELECTION_BG: Color = Color::DarkGray;

/// Color for table headers
pub const COLOR_HEADER: Color = Color::Yellow;

/// Color for help text and secondary information
pub const COLOR_HELP_TEXT: Color = Color::Gray;

/// Color for screen titles and accent text
pub const COLOR_TITLE: Color = Color::Cyan;

/// Color for loading/status messages
pub const COLOR_LOADING: Color = Color::Yellow;

/// Border color for danger/warning popups (delete confirmations)
pub const COLOR_BORDER_DANGER: Color = Color::Red;

/// Border color for informational popups
pub const COLOR_BORDER_INFO: Color = Color::Blue;

/// Border color for accent/highlighted elements
pub const COLOR_BORDER_ACCENT: Color = Color::Cyan;

/// Color for input fields when focused
pub const COLOR_INPUT_FOCUSED: Color = Color::Yellow;

/// Background for form fields when focused
pub const COLOR_FORM_FIELD_BG: Color = Color::DarkGray;

// =============================================================================
// Layout Constants
// =============================================================================

/// Standard margin around screen content
pub const SCREEN_MARGIN: u16 = 2;

/// Height of the title/header area
pub const TITLE_HEIGHT: u16 = 1;

/// Height of the help bar at the bottom
pub const HELP_BAR_HEIGHT: u16 = 3;

/// Height of filter input when visible
pub const FILTER_INPUT_HEIGHT: u16 = 3;

/// Standard column spacing for tables
pub const TABLE_COLUMN_SPACING: u16 = 2;

/// Width of sidebar in plan screen
pub const SIDEBAR_WIDTH: u16 = 16;

/// Height of summary cards
pub const SUMMARY_CARD_HEIGHT: u16 = 3;

// =============================================================================
// Style Functions
// =============================================================================

/// Style for selected/highlighted rows in tables and lists
pub fn selection_style() -> Style {
    Style::default()
        .bg(COLOR_SELECTION_BG)
        .add_modifier(Modifier::BOLD)
}

/// Style for table headers
pub fn header_style() -> Style {
    Style::default()
        .fg(COLOR_HEADER)
        .add_modifier(Modifier::BOLD)
}

/// Style for help bar text
pub fn help_text_style() -> Style {
    Style::default().fg(COLOR_HELP_TEXT)
}

/// Style for screen titles
pub fn title_style() -> Style {
    Style::default()
        .fg(COLOR_TITLE)
        .add_modifier(Modifier::BOLD)
}

/// Style for loading/status messages
pub fn loading_style() -> Style {
    Style::default().fg(COLOR_LOADING)
}

/// Style for form fields when focused
pub fn form_field_focused_style() -> Style {
    Style::default()
        .bg(COLOR_FORM_FIELD_BG)
        .add_modifier(Modifier::BOLD)
}

/// Style for form fields when not focused
pub fn form_field_style() -> Style {
    Style::default().fg(Color::White)
}

/// Style for danger/warning borders (delete confirmations)
pub fn danger_border_style() -> Style {
    Style::default()
        .fg(COLOR_BORDER_DANGER)
        .add_modifier(Modifier::BOLD)
}

/// Style for info borders
pub fn info_border_style() -> Style {
    Style::default()
        .fg(COLOR_BORDER_INFO)
        .add_modifier(Modifier::BOLD)
}

/// Style for accent borders
pub fn accent_border_style() -> Style {
    Style::default().fg(COLOR_BORDER_ACCENT)
}

// =============================================================================
// Amount Color Helper
// =============================================================================

/// Get the appropriate color for an amount value.
/// Positive = green, negative = red, zero = gray
pub fn amount_color(amount: i64) -> Color {
    if amount > 0 {
        COLOR_POSITIVE
    } else if amount < 0 {
        COLOR_NEGATIVE
    } else {
        COLOR_ZERO
    }
}

/// Get the appropriate color for a float amount value.
/// Positive = green, negative = red, zero = gray
pub fn amount_color_f64(amount: f64) -> Color {
    if amount > 0.0 {
        COLOR_POSITIVE
    } else if amount < 0.0 {
        COLOR_NEGATIVE
    } else {
        COLOR_ZERO
    }
}
