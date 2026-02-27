use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::state::{AccountsState, InputMode, LoadingState};
use crate::ui::{
    components::{empty_state, filter_input, help_bar, screen_title},
    layouts, theme, utils,
};
use ynab_api::endpoints::{accounts::AccountType, budgets::BudgetSummary};

pub fn render(f: &mut Frame, state: &AccountsState, budget: Option<&BudgetSummary>) {
    if state.input_mode == InputMode::Filter {
        let (title_area, filter_area, content_area, help_area) =
            layouts::screen_layout_with_filter(f.area());

        screen_title::render_screen_title(f, title_area, &state.accounts_loading);
        filter_input::render_filter_input(f, filter_area, &state.filter_query);
        render_content(f, content_area, state, budget);
        help_bar::render_help_bar(f, help_area, help_bar::HELP_TEXT_DEFAULT);
    } else {
        let (title_area, content_area, help_area) = layouts::screen_layout(f.area());

        screen_title::render_screen_title(f, title_area, &state.accounts_loading);
        render_content(f, content_area, state, budget);
        help_bar::render_help_bar(f, help_area, help_bar::HELP_TEXT_DEFAULT);
    }
}

fn render_content(
    f: &mut Frame,
    area: Rect,
    state: &AccountsState,
    budget: Option<&BudgetSummary>,
) {
    // Show loading message if currently loading and no cached data
    if matches!(state.accounts_loading, LoadingState::Loading(..)) && state.accounts.is_empty() {
        empty_state::render_loading_state(f, area, "Status", "Loading accounts...");
        return;
    }

    // Apply filter to accounts
    let filtered = state.filtered_accounts();

    // Show accounts table if we have data
    if !filtered.is_empty() {
        // Create table header
        let header = Row::new(vec![
            Cell::from("Account Name"),
            Cell::from("Type"),
            Cell::from(Text::from("Balance").right_aligned()),
        ])
        .style(theme::header_style())
        .underlined();

        // Create table rows from filtered accounts
        let rows: Vec<Row> = filtered
            .iter()
            .map(|account| {
                let balance_color = utils::get_amount_color(account.balance.into());
                let balance_str = utils::format_amount(account.balance.into(), budget);

                Row::new(vec![
                    Cell::from(account.name.clone()),
                    Cell::from(format_account_type(account.account_type)),
                    Cell::from(Text::from(balance_str).right_aligned())
                        .style(Style::default().fg(balance_color)),
                ])
            })
            .collect();

        // Update table title to show filter status
        let title = if !state.filter_query.is_empty() {
            format!("Accounts ({} filtered)", filtered.len())
        } else {
            "Accounts".to_string()
        };

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(theme::selection_style());

        f.render_stateful_widget(table, area, &mut state.table_state.borrow_mut());
    } else {
        // No matching accounts - show message based on filter state
        let message = if !state.filter_query.is_empty() {
            "No matching accounts"
        } else {
            "No accounts found"
        };

        empty_state::render_empty_state(f, area, "Accounts", message, None);
    }
}

fn format_account_type(account_type: AccountType) -> &'static str {
    use AccountType::*;
    match account_type {
        Checking | Savings | Cash => "Cash",
        CreditCard | LineOfCredit => "Credit",
        Mortgage | AutoLoan | StudentLoan | PersonalLoan | MedicalDebt | OtherDebt => "Debt",
        OtherAsset | OtherLiability => "Tracking",
    }
}
