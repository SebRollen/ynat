//! Shared screen title component with loading indicator.

use ratatui::prelude::Rect;
use ratatui::Frame;

use crate::state::LoadingState;
use crate::ui::layouts;

use super::loading_indicator;

/// Render a screen title area with loading indicator on the right.
///
/// This provides consistent title styling with the loading spinner
/// positioned in the top-right corner.
pub fn render_screen_title(f: &mut Frame, area: Rect, loading_state: &LoadingState) {
    let (_, indicator_area) = layouts::title_with_loading(area);

    // Render loading indicator in the right portion
    loading_indicator::render_loading_indicator(f, indicator_area, loading_state);
}
