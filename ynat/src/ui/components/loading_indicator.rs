use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};

use crate::state::LoadingState;

/// Render a loading indicator in the top-right corner
/// Shows current loading state with color coding
pub fn render_loading_indicator(f: &mut Frame, area: Rect, loading_state: &LoadingState) {
    let (text, color) = match &loading_state {
        LoadingState::NotStarted => return, // Don't show anything
        LoadingState::Loading(throbber_state) => {
            let simple = throbber_widgets_tui::Throbber::default()
                .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT);
            f.render_stateful_widget(simple, area, &mut throbber_state.clone());
            return;
        }
        LoadingState::Loaded => ("✓", Color::Green),
        LoadingState::Error(_) => ("x", Color::Red),
    };

    let indicator =
        Paragraph::new(Span::styled(text, Style::default().fg(color))).alignment(Alignment::Right);

    f.render_widget(indicator, area);
}
