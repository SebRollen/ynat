//! Shared help bar component for consistent bottom navigation hints.

use ratatui::prelude::Rect;
use ratatui::{
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme;

/// Render a standard help bar with the given text.
///
/// The help bar is styled consistently with gray text in a bordered block,
/// centered alignment. All screens should use this for their help bar.
pub fn render_help_bar(f: &mut Frame, area: Rect, text: &str) {
    let help = Paragraph::new(text)
        .style(theme::help_text_style())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(help, area);
}

/// Standard help bar text used across most screens
pub const HELP_TEXT_DEFAULT: &str = "Press ? for help";
