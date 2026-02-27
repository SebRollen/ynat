use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Row, Table},
};
use tracing::Level;

use crate::log_buffer::LogBuffer;
use crate::state::LogsState;
use crate::ui::{
    components::{empty_state, help_bar},
    layouts, theme,
};

pub fn render(f: &mut Frame, state: &LogsState, log_buffer: &LogBuffer) {
    let (title_area, content_area, help_area) = layouts::screen_layout(f.area());

    render_title(f, title_area, state);
    render_logs(f, content_area, state, log_buffer);
    render_help(f, help_area, state);
}

fn render_title(f: &mut Frame, area: Rect, state: &LogsState) {
    let title = format!("Logs ({} entries)", state.total_entries);
    let paragraph = ratatui::widgets::Paragraph::new(title).style(theme::title_style());
    f.render_widget(paragraph, area);
}

fn render_logs(f: &mut Frame, area: Rect, state: &LogsState, log_buffer: &LogBuffer) {
    let entries = log_buffer.get_entries();
    let total = entries.len();

    if total == 0 {
        empty_state::render_empty_state(f, area, "Session Logs", "No logs yet", None);
        return;
    }

    // Calculate visible window (scrolling from bottom, newest at bottom)
    let inner_height = area.height.saturating_sub(2) as usize; // Account for borders
    let start = total.saturating_sub(state.scroll_offset + inner_height);
    let end = total.saturating_sub(state.scroll_offset);

    let rows: Vec<Row> = entries[start..end]
        .iter()
        .map(|entry| {
            let level_style = match entry.level {
                Level::ERROR => Style::default()
                    .fg(theme::COLOR_NEGATIVE)
                    .add_modifier(Modifier::BOLD),
                Level::WARN => Style::default().fg(theme::COLOR_LOADING),
                Level::INFO => Style::default().fg(theme::COLOR_POSITIVE),
                Level::DEBUG => Style::default().fg(Color::Blue),
                Level::TRACE => Style::default().fg(theme::COLOR_ZERO),
            };

            let level_str = match entry.level {
                Level::ERROR => "ERROR",
                Level::WARN => "WARN ",
                Level::INFO => "INFO ",
                Level::DEBUG => "DEBUG",
                Level::TRACE => "TRACE",
            };

            Row::new(vec![
                entry.timestamp.format("%H:%M:%S%.3f").to_string(),
                level_str.to_string(),
                truncate_target(&entry.target, 25),
                entry.message.clone(),
            ])
            .style(level_style)
        })
        .collect();

    let widths = [
        Constraint::Length(12), // Time
        Constraint::Length(5),  // Level
        Constraint::Length(25), // Target
        Constraint::Min(30),    // Message
    ];

    let table = Table::new(rows, widths)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Logs [{}-{} of {}] ",
            start + 1,
            end,
            total
        )))
        .header(
            Row::new(vec!["Time", "Level", "Target", "Message"])
                .style(theme::header_style())
                .bottom_margin(1),
        );

    f.render_widget(table, area);
}

fn render_help(f: &mut Frame, area: Rect, state: &LogsState) {
    let scroll_info = if state.scroll_offset > 0 {
        format!(" (scrolled {} from bottom)", state.scroll_offset)
    } else {
        String::new()
    };

    let help_text = format!(
        "j/k: scroll | G: bottom | gg: top | PgUp/PgDn: page | h: back | ?: help{}",
        scroll_info
    );

    help_bar::render_help_bar(f, area, &help_text);
}

fn truncate_target(target: &str, max_len: usize) -> String {
    if target.len() <= max_len {
        target.to_string()
    } else {
        format!("...{}", &target[target.len() - max_len + 3..])
    }
}
