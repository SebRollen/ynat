use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};

use crate::state::{BudgetsState, LoadingState};
use crate::ui::{
    components::{empty_state, help_bar, screen_title},
    layouts, theme,
};

pub fn render(f: &mut Frame, state: &BudgetsState) {
    let (title_area, content_area, help_area) = layouts::screen_layout(f.area());

    screen_title::render_screen_title(f, title_area, &state.budgets_loading);
    render_content(f, content_area, state);
    help_bar::render_help_bar(f, help_area, help_bar::HELP_TEXT_DEFAULT);
}

fn render_content(f: &mut Frame, area: Rect, state: &BudgetsState) {
    // Show loading message if currently loading and no cached data
    if matches!(state.budgets_loading, LoadingState::Loading(..)) && state.budgets.is_empty() {
        empty_state::render_loading_state(f, area, "Status", "Loading budgets...");
        return;
    }

    // Show budgets list if we have data
    if !state.budgets.is_empty() {
        let items: Vec<ListItem> = state
            .budgets
            .iter()
            .enumerate()
            .map(|(i, budget)| {
                let style = if i == state.selected_budget_index {
                    theme::selection_style()
                } else {
                    Style::default()
                };

                ListItem::new(budget.name.clone()).style(style)
            })
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Budgets"));

        f.render_widget(list, area);
    } else {
        // No budgets - show empty message
        empty_state::render_empty_state(
            f,
            area,
            "Budgets",
            "No budgets found",
            Some("Create a budget at https://app.ynab.com"),
        );
    }
}
