use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::{layouts, theme};

/// Render a confirmation popup for transaction deletion
pub fn render_delete_confirmation(f: &mut Frame) {
    // Create centered popup using shared layout helper
    let inner = super::popup::render_popup_frame(
        f,
        f.area(),
        layouts::popup_sizes::SMALL,
        " Confirm Delete ",
        theme::danger_border_style(),
    );

    // Create content layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Warning message
            Constraint::Length(1), // Empty line
            Constraint::Length(1), // Instructions
        ])
        .split(inner);

    // Warning message
    let warning = Paragraph::new("Are you sure you want to delete this transaction?")
        .style(theme::loading_style().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(warning, chunks[0]);

    // Instructions
    let instructions = Line::from(vec![
        Span::styled(
            "[Y]es ",
            Style::default()
                .fg(theme::COLOR_POSITIVE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("/ "),
        Span::styled(
            "[N]o ",
            Style::default()
                .fg(theme::COLOR_NEGATIVE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("/ "),
        Span::styled("[Esc]", Style::default().fg(theme::COLOR_HELP_TEXT)),
        Span::raw(" Cancel"),
    ]);
    let instructions_para = Paragraph::new(instructions).alignment(Alignment::Center);
    f.render_widget(instructions_para, chunks[2]);
}
