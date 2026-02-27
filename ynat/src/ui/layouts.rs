//! Reusable layout builders for consistent screen structure.
//!
//! These functions provide standard layouts that all screens should use
//! to ensure consistent margins, spacing, and element positioning.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

use super::theme::{FILTER_INPUT_HEIGHT, HELP_BAR_HEIGHT, SCREEN_MARGIN, TITLE_HEIGHT};

/// Standard screen layout with title, content area, and help bar.
///
/// Returns a tuple of (title_area, content_area, help_area)
pub fn screen_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(SCREEN_MARGIN)
        .constraints([
            Constraint::Length(TITLE_HEIGHT),
            Constraint::Min(10),
            Constraint::Length(HELP_BAR_HEIGHT),
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2])
}

/// Screen layout with filter input visible.
///
/// Returns a tuple of (title_area, filter_area, content_area, help_area)
pub fn screen_layout_with_filter(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(SCREEN_MARGIN)
        .constraints([
            Constraint::Length(TITLE_HEIGHT),
            Constraint::Length(FILTER_INPUT_HEIGHT),
            Constraint::Min(10),
            Constraint::Length(HELP_BAR_HEIGHT),
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2], chunks[3])
}

/// Split a title area into title text and loading indicator.
///
/// Returns (title_text_area, loading_indicator_area)
pub fn title_with_loading(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100), Constraint::Length(1)])
        .split(area);

    (chunks[0], chunks[1])
}

/// Create a centered popup rectangle.
///
/// # Arguments
/// * `percent_x` - Width as percentage of parent (0-100)
/// * `percent_y` - Height as percentage of parent (0-100)
/// * `area` - The parent area to center within
pub fn centered_popup(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Screen layout with left sidebar.
///
/// Returns a tuple of (header_area, sidebar_area, main_area, help_area)
pub fn screen_layout_with_sidebar(area: Rect, sidebar_width: u16) -> (Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(SCREEN_MARGIN)
        .constraints([
            Constraint::Length(TITLE_HEIGHT),
            Constraint::Min(10),
            Constraint::Length(HELP_BAR_HEIGHT),
        ])
        .split(area);

    let header_area = chunks[0];
    let help_area = chunks[2];

    // Split the content area into sidebar and main
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(sidebar_width), Constraint::Min(20)])
        .split(chunks[1]);

    (header_area, content_chunks[0], content_chunks[1], help_area)
}

/// Standard popup sizes
pub mod popup_sizes {
    /// Small popup (50% x 30%) - for simple confirmations
    pub const SMALL: (u16, u16) = (50, 30);

    /// Medium popup (60% x 30%) - for confirmations with more info
    pub const MEDIUM: (u16, u16) = (60, 30);

    /// Large popup (80% x 80%) - for help screens and complex dialogs
    pub const LARGE: (u16, u16) = (80, 80);
}
