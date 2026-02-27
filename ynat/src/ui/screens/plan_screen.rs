use crate::state::{InputMode, LoadingState, PlanFocusedView, PlanState};
use crate::ui::{
    components::{empty_state, help_bar, loading_indicator},
    layouts, theme, utils,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Row, Table},
};
use ynab_api::endpoints::{budgets::BudgetSummary, months::MonthDetail};

pub fn render(f: &mut Frame, state: &PlanState, budget: Option<&BudgetSummary>) {
    let area = f.area();

    // Use sidebar layout
    let (header_area, sidebar_area, main_area, help_area) =
        layouts::screen_layout_with_sidebar(area, theme::SIDEBAR_WIDTH);

    render_header(f, header_area, state);
    render_sidebar(f, sidebar_area, state);
    render_main_content(f, main_area, state, budget);
    help_bar::render_help_bar(
        f,
        help_area,
        "j/k: navigate  e: edit  ,: view  Tab: month  ?: help",
    );
}

fn render_header(f: &mut Frame, area: Rect, state: &PlanState) {
    // Format month nicely (e.g., "January 2025")
    let month_display = if let Some(month) = &state.month {
        format_month_display(&month.month)
    } else {
        "Plan".to_string()
    };

    // Create header with month on left, navigation hint on right
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),    // Month title
            Constraint::Length(28), // Loading indicator
            Constraint::Length(16), // Navigation hint
        ])
        .split(area);

    // Month title
    let title = Paragraph::new(month_display).style(theme::title_style());
    f.render_widget(title, header_chunks[0]);

    // Loading indicator
    loading_indicator::render_loading_indicator(f, header_chunks[1], &state.plan_loading);

    // Navigation hint
    let nav_hint = Paragraph::new("◀ S-Tab │ Tab ▶")
        .style(Style::default().fg(theme::COLOR_HELP_TEXT))
        .alignment(Alignment::Right);
    f.render_widget(nav_hint, header_chunks[2]);
}

fn render_sidebar(f: &mut Frame, area: Rect, state: &PlanState) {
    let block = Block::default().borders(Borders::RIGHT);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Build sidebar content
    let views = [
        (PlanFocusedView::All, "All"),
        (PlanFocusedView::Underfunded, "Underfunded"),
        (PlanFocusedView::Overfunded, "Overfunded"),
        (PlanFocusedView::Snoozed, "Snoozed"),
        (PlanFocusedView::MoneyAvailable, "Available"),
    ];

    let mut lines = vec![
        Line::from(Span::styled(
            "VIEWS",
            Style::default()
                .fg(theme::COLOR_HEADER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("──────────────"),
    ];

    for (view, label) in views {
        let prefix = if state.focused_view == view {
            "▸ "
        } else {
            "  "
        };
        let style = if state.focused_view == view {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::COLOR_HELP_TEXT)
        };
        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, label),
            style,
        )));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);
}

fn render_main_content(
    f: &mut Frame,
    area: Rect,
    state: &PlanState,
    budget: Option<&BudgetSummary>,
) {
    // Show loading message if currently loading and no cached data
    if matches!(state.plan_loading, LoadingState::Loading(..)) && state.categories.is_empty() {
        empty_state::render_loading_state(f, area, "Status", "Loading plan...");
        return;
    }

    // Show month summary if available
    if let Some(month) = &state.month {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(theme::SUMMARY_CARD_HEIGHT),
                Constraint::Min(0),
            ])
            .split(area);

        render_summary_cards(f, chunks[0], month, budget);
        render_categories_table(f, chunks[1], state);
    } else {
        // No data loaded yet
        empty_state::render_empty_state(
            f,
            area,
            "Plan",
            "No plan data loaded",
            Some("Press 'r' to load"),
        );
    }
}

fn render_summary_cards(
    f: &mut Frame,
    area: Rect,
    month: &MonthDetail,
    budget: Option<&BudgetSummary>,
) {
    // Split into 3 equal cards with gaps
    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    // Card 1: To Be Budgeted
    let tbb_str = utils::format_amount(month.to_be_budgeted.into(), budget);
    render_card(
        f,
        card_chunks[0],
        &tbb_str,
        "To Budget",
        utils::get_amount_color(month.to_be_budgeted.into()),
    );

    // Card 2: Income
    let income_str = utils::format_amount(month.income.into(), budget);
    render_card(
        f,
        card_chunks[1],
        &income_str,
        "Income",
        utils::get_amount_color(month.income.into()),
    );

    // Card 3: Budgeted
    let budgeted_str = utils::format_amount(month.budgeted.into(), budget);
    render_card(
        f,
        card_chunks[2],
        &budgeted_str,
        "Budgeted",
        utils::get_amount_color(month.budgeted.into()),
    );
}

fn render_card(f: &mut Frame, area: Rect, amount: &str, label: &str, color: Color) {
    let block = Block::default().borders(Borders::ALL).title(label);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let paragraph = Paragraph::new(Span::styled(
        amount,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center);
    f.render_widget(paragraph, inner);
}

fn render_categories_table(f: &mut Frame, area: Rect, state: &PlanState) {
    // Use filtered categories based on focused view
    let visible_categories = state.filtered_categories();

    // Generate title based on focused view (used for both empty state and table)
    let title = match state.focused_view {
        PlanFocusedView::All => "Categories".to_string(),
        _ => format!("Categories - {}", state.focused_view.display_name()),
    };

    if visible_categories.is_empty() {
        let message = match state.focused_view {
            PlanFocusedView::All => "No categories to display",
            PlanFocusedView::Snoozed => "No snoozed categories",
            PlanFocusedView::Underfunded => "No underfunded categories",
            PlanFocusedView::Overfunded => "No overfunded categories",
            PlanFocusedView::MoneyAvailable => "No categories with money available",
        };
        empty_state::render_empty_state(f, area, &title, message, None);
        return;
    }

    // Get editing category ID if in edit mode
    let editing_category_id = if state.input_mode == InputMode::BudgetEdit {
        state.budget_form.as_ref().map(|f| f.category_id.as_str())
    } else {
        None
    };

    // Create table rows
    let rows: Vec<Row> = visible_categories
        .iter()
        .map(|category| {
            // Convert milliunits to dollars
            let budgeted = category.budgeted.as_f64() / 1000.0;
            let activity = category.activity.as_f64() / 1000.0;
            let balance = category.balance.as_f64() / 1000.0;

            // Check if this category is being edited
            let budgeted_cell =
                if editing_category_id.map(|s| s.to_string()) == Some(category.id.to_string()) {
                    // Show inline edit field
                    if let Some(ref form) = state.budget_form {
                        let input_text = if form.budgeted_input.is_empty() {
                            "_______".to_string()
                        } else {
                            format!("{}_", form.budgeted_input)
                        };
                        Text::from(input_text)
                            .style(theme::form_field_focused_style().fg(Color::White))
                            .right_aligned()
                    } else {
                        Text::from(utils::fmt_dollars(budgeted))
                            .style(Style::default().fg(utils::get_amount_color_f64(budgeted)))
                            .right_aligned()
                    }
                } else {
                    Text::from(utils::fmt_dollars(budgeted))
                        .style(Style::default().fg(utils::get_amount_color_f64(budgeted)))
                        .right_aligned()
                };

            Row::new(vec![
                Text::from(category.name.clone()),
                budgeted_cell,
                Text::from(utils::fmt_dollars(activity))
                    .style(Style::default().fg(utils::get_amount_color_f64(activity)))
                    .right_aligned(),
                Text::from(utils::fmt_dollars(balance))
                    .style(Style::default().fg(utils::get_amount_color_f64(balance)))
                    .right_aligned(),
            ])
        })
        .collect();

    // Create header
    let header = Row::new(vec![
        Text::from("Category"),
        Text::from("Budgeted").right_aligned(),
        Text::from("Activity").right_aligned(),
        Text::from("Available").right_aligned(),
    ])
    .style(theme::header_style())
    .underlined();

    // Override title if in edit mode
    let title = if state.input_mode == InputMode::BudgetEdit {
        if let Some(ref form) = state.budget_form {
            if let Some(ref error) = form.validation_error {
                format!("Categories - {} [{}]", error, form.category_name)
            } else {
                format!(
                    "Categories - Editing: {} (Enter=save, Esc=cancel)",
                    form.category_name
                )
            }
        } else {
            title
        }
    } else {
        title
    };

    // Create table
    let mut table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));

    // Only highlight rows when not in edit mode
    if state.input_mode != InputMode::BudgetEdit {
        table = table.row_highlight_style(theme::selection_style());
    }

    f.render_stateful_widget(table, area, &mut state.table_state.borrow_mut());
}

/// Format a month string (YYYY-MM-DD) to a human-readable format (e.g., "January 2025")
fn format_month_display(month: &str) -> String {
    // Parse YYYY-MM-DD format
    let parts: Vec<&str> = month.split('-').collect();
    if parts.len() >= 2 {
        let year = parts[0];
        let month_num: u32 = parts[1].parse().unwrap_or(1);
        let month_name = match month_num {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        };
        format!("{} {}", month_name, year)
    } else {
        month.to_string()
    }
}
