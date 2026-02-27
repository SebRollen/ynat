//! Shared filter input component for screens with filtering capability.

use ratatui::prelude::Rect;
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme;

/// Render a filter input field with the current query.
///
/// This provides a consistent filter UI across screens (Accounts, Transactions).
pub fn render_filter_input(f: &mut Frame, area: Rect, query: &str) {
    let input = Paragraph::new(query)
        .style(theme::loading_style()) // Yellow text for input
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Filter (Enter: apply, Esc: clear)"),
        );

    f.render_widget(input, area);
}
