//! Shared popup/modal base component.

use ratatui::prelude::Rect;
use ratatui::{
    layout::Alignment,
    style::Style,
    widgets::{Block, Borders, Clear},
    Frame,
};

use crate::ui::layouts;

/// Render a popup frame and return the inner area for content.
///
/// This handles:
/// - Centering the popup
/// - Clearing the background
/// - Drawing the border with title
///
/// # Arguments
/// * `size` - Tuple of (width_percent, height_percent)
/// * `title` - The popup title
/// * `border_style` - Style for the border (use theme::danger_border_style(), etc.)
///
/// # Returns
/// The inner area where popup content should be rendered
pub fn render_popup_frame(
    f: &mut Frame,
    parent_area: Rect,
    size: (u16, u16),
    title: &str,
    border_style: Style,
) -> Rect {
    let area = layouts::centered_popup(size.0, size.1, parent_area);

    // Clear the background
    f.render_widget(Clear, area);

    // Create and render the border
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    inner
}
