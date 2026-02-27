use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use crate::ui::theme;

/// A text input widget with an autocomplete dropdown overlay
pub struct AutocompleteInput<'a> {
    /// The current input value
    pub value: &'a str,
    /// Placeholder text when value is empty
    pub placeholder: &'a str,
    /// Whether the input is focused
    pub is_focused: bool,
    /// Autocomplete items to show in dropdown
    pub items: &'a [String],
    /// Currently selected item index
    pub selected_index: usize,
    /// Optional hint text at bottom of dropdown
    pub hint: Option<&'a str>,
}

impl<'a> AutocompleteInput<'a> {
    pub fn new(value: &'a str, placeholder: &'a str) -> Self {
        Self {
            value,
            placeholder,
            is_focused: false,
            items: &[],
            selected_index: 0,
            hint: None,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn items(mut self, items: &'a [String]) -> Self {
        self.items = items;
        self
    }

    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    pub fn hint(mut self, hint: Option<&'a str>) -> Self {
        self.hint = hint;
        self
    }

    /// Render the input and dropdown overlay
    /// Returns the area used by the input (for reference)
    pub fn render(self, f: &mut Frame, area: Rect) -> Rect {
        // Render the input value
        let style = if self.is_focused {
            theme::form_field_focused_style()
        } else {
            theme::form_field_style()
        };

        let display_value = if self.value.is_empty() {
            self.placeholder
        } else {
            self.value
        };

        // Clear and render input area
        f.render_widget(Clear, area);
        f.render_widget(Span::from(display_value).style(style), area);

        // Render dropdown if we have items or a hint to show
        if !self.items.is_empty() || self.hint.is_some() {
            self.render_dropdown(f, area);
        }

        area
    }

    fn render_dropdown(&self, f: &mut Frame, input_area: Rect) {
        let item_count = self.items.len();
        let has_hint = self.hint.is_some();
        let content_height = if has_hint { item_count + 1 } else { item_count };

        if content_height == 0 {
            return;
        }

        // Dropdown dimensions (add 2 for borders)
        let dropdown_height = (content_height + 2) as u16;
        let dropdown_width = input_area.width.max(20);

        // Position dropdown below the input
        let x = input_area.x;
        let y = input_area.y + 1;

        // Ensure we don't go off screen
        let frame_height = f.area().height;
        let frame_width = f.area().width;

        let (final_y, final_height) = if y + dropdown_height > frame_height {
            // Not enough room below, position above if possible
            if input_area.y >= dropdown_height {
                (
                    input_area.y.saturating_sub(dropdown_height),
                    dropdown_height,
                )
            } else {
                // Truncate to fit
                (y, frame_height.saturating_sub(y).max(3))
            }
        } else {
            (y, dropdown_height)
        };

        let final_width = dropdown_width.min(frame_width.saturating_sub(x));
        let dropdown_area = Rect::new(x, final_y, final_width, final_height);

        // Clear the dropdown area
        f.render_widget(Clear, dropdown_area);

        // Build list items
        let mut list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let is_selected = i == self.selected_index;
                let style = if is_selected {
                    theme::selection_style()
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::from(name.clone()).style(style)))
            })
            .collect();

        // Add hint if present
        if let Some(hint) = self.hint {
            list_items.push(ListItem::new(Line::from(
                Span::from(hint).style(Style::default().fg(Color::DarkGray)),
            )));
        }

        let list = List::new(list_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(list, dropdown_area);
    }
}
