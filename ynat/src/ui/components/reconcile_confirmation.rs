use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::ui::{layouts, theme, utils};
use ynab_api::endpoints::CurrencyFormat;

/// Render a confirmation popup for account reconciliation
pub fn render_reconcile_confirmation(
    f: &mut Frame,
    cleared_balance: i64,
    currency_format: Option<&CurrencyFormat>,
) {
    // Create centered popup using shared layout helper
    let inner = super::popup::render_popup_frame(
        f,
        f.area(),
        layouts::popup_sizes::MEDIUM,
        " Reconcile Account ",
        theme::info_border_style(),
    );

    // Create content layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Question
            Constraint::Length(1), // Empty line
            Constraint::Length(1), // Cleared balance
            Constraint::Length(1), // Empty line
            Constraint::Length(2), // Instructions
        ])
        .split(inner);

    // Question
    let question = Paragraph::new("Does your current account balance match the cleared balance?")
        .style(
            Style::default()
                .fg(ratatui::style::Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(question, chunks[0]);

    // Format the cleared balance
    let balance_value = cleared_balance as f64 / 1000.0;
    let formatted_balance = if let Some(fmt) = currency_format {
        utils::fmt_currency(cleared_balance, fmt)
            .content
            .to_string()
    } else {
        format!("${:.2}", balance_value)
    };

    let balance_color = theme::amount_color_f64(balance_value);

    let balance_text = Paragraph::new(format!("Cleared balance: {}", formatted_balance))
        .style(
            Style::default()
                .fg(balance_color)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(balance_text, chunks[2]);

    // Instructions
    let instructions = Line::from(vec![
        Span::styled(
            "[Y]es ",
            Style::default()
                .fg(theme::COLOR_POSITIVE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("- Mark cleared transactions as reconciled / "),
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
    f.render_widget(instructions_para, chunks[4]);
}
