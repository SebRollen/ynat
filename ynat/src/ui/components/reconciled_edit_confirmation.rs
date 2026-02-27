use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::{layouts, theme};

/// Render a confirmation popup for editing reconciled transactions
pub fn render_reconciled_edit_confirmation(f: &mut Frame) {
    // Use shared popup frame with warning style
    let inner = super::popup::render_popup_frame(
        f,
        f.area(),
        layouts::popup_sizes::MEDIUM,
        " Edit Reconciled Transaction ",
        theme::loading_style().add_modifier(Modifier::BOLD), // Yellow for warning
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Warning message
            Constraint::Length(1), // Empty line
            Constraint::Length(2), // Explanation
            Constraint::Length(1), // Empty line
            Constraint::Length(1), // Instructions
        ])
        .split(inner);

    // Warning message
    let warning = Paragraph::new(
        "This transaction is marked as RECONCILED.\nAre you sure you want to edit it?",
    )
    .style(theme::loading_style().add_modifier(Modifier::BOLD))
    .alignment(Alignment::Center);
    f.render_widget(warning, chunks[0]);

    // Explanation
    let explanation = Paragraph::new(
        "Editing reconciled transactions may cause discrepancies\nwith your bank records.",
    )
    .style(
        Style::default()
            .fg(theme::COLOR_HELP_TEXT)
            .add_modifier(Modifier::ITALIC),
    )
    .alignment(Alignment::Center);
    f.render_widget(explanation, chunks[2]);

    // Instructions
    let instructions = Line::from(vec![
        Span::styled(
            "[Y]es, Edit ",
            Style::default()
                .fg(theme::COLOR_POSITIVE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("/ "),
        Span::styled(
            "[N]o, Cancel ",
            Style::default()
                .fg(theme::COLOR_NEGATIVE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("/ "),
        Span::styled("[Esc]", Style::default().fg(theme::COLOR_HELP_TEXT)),
    ]);
    let instructions_para = Paragraph::new(instructions).alignment(Alignment::Center);
    f.render_widget(instructions_para, chunks[4]);
}
