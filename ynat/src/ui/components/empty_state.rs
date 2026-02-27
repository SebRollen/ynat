//! Shared empty state component for consistent "no data" messages.

use ratatui::prelude::Rect;
use ratatui::{
    layout::Alignment,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme;

/// Render an empty state message with consistent styling.
///
/// Used when a list/table has no data to display.
///
/// # Arguments
/// * `title` - The block title (e.g., "Budgets", "Transactions")
/// * `message` - The message to display (e.g., "No budgets found")
/// * `hint` - Optional hint text below the message
pub fn render_empty_state(
    f: &mut Frame,
    area: Rect,
    title: &str,
    message: &str,
    hint: Option<&str>,
) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(message, theme::loading_style())),
    ];

    if let Some(hint_text) = hint {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            hint_text,
            theme::help_text_style(),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(paragraph, area);
}

/// Render a loading state message with consistent styling.
///
/// Used when data is being loaded and no cached data exists.
pub fn render_loading_state(f: &mut Frame, area: Rect, title: &str, message: &str) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(message, theme::loading_style())),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(paragraph, area);
}
