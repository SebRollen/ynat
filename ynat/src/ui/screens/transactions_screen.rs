use ratatui::{
    prelude::*,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::state::{InputMode, LoadingState, TransactionsState};
use crate::ui::{
    components::{empty_state, filter_input, help_bar, inline_transaction_form, screen_title},
    layouts, theme, utils,
};
use itertools::Itertools;
use ynab_api::endpoints::{
    budgets::BudgetSummary,
    transactions::{ReconciliationStatus, SubTransaction, Transaction},
};

pub fn render(f: &mut Frame, state: &TransactionsState, budget: Option<&BudgetSummary>) {
    if state.input_mode == InputMode::Filter {
        let (title_area, filter_area, content_area, help_area) =
            layouts::screen_layout_with_filter(f.area());

        screen_title::render_screen_title(f, title_area, &state.transactions_loading);
        filter_input::render_filter_input(f, filter_area, &state.filter_query);
        render_content(f, content_area, state, budget);
        help_bar::render_help_bar(f, help_area, help_bar::HELP_TEXT_DEFAULT);
    } else {
        let (title_area, content_area, help_area) = layouts::screen_layout(f.area());

        screen_title::render_screen_title(f, title_area, &state.transactions_loading);
        render_content(f, content_area, state, budget);
        help_bar::render_help_bar(f, help_area, help_bar::HELP_TEXT_DEFAULT);
    }
}

fn render_content(
    f: &mut Frame,
    area: Rect,
    state: &TransactionsState,
    budget: Option<&BudgetSummary>,
) {
    // Show loading message if currently loading and no cached data
    if matches!(state.transactions_loading, LoadingState::Loading(..))
        && state.transactions.is_empty()
    {
        empty_state::render_loading_state(f, area, "Status", "Loading transactions...");
        return;
    }

    // Check if we have a validation error to display
    let has_error = state
        .form_state
        .as_ref()
        .map(inline_transaction_form::has_validation_error)
        .unwrap_or(false);

    // Build layout with optional error row
    let (summary_area, error_area, table_area) = if has_error {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(theme::SUMMARY_CARD_HEIGHT),
                Constraint::Length(1), // Error row
                Constraint::Min(0),
            ])
            .split(area);
        (chunks[0], Some(chunks[1]), chunks[2])
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(theme::SUMMARY_CARD_HEIGHT),
                Constraint::Min(0),
            ])
            .split(area);
        (chunks[0], None, chunks[1])
    };

    // Render balance summary in the top area
    render_balance_summary(f, summary_area, state, budget);

    // Render validation error if present
    if let (Some(error_rect), Some(ref form_state)) = (error_area, &state.form_state) {
        inline_transaction_form::render_validation_error(f, error_rect, form_state);
    }

    // Apply filter to transactions
    let filtered = state.filtered_transactions();

    // Show transactions table if we have data
    if !filtered.is_empty() {
        // Create table header
        let header = Row::new(vec![
            Cell::from("▱"),
            Cell::from("Date"),
            Cell::from("Payee"),
            Cell::from("Category"),
            Cell::from("Memo"),
            Cell::from(Text::from("Amount").right_aligned()),
            Cell::from("ⓘ"),
            Cell::from("C"),
        ])
        .style(theme::header_style())
        .underlined();

        // Create table rows from filtered transactions
        // Track form visual Y offset for direct rendering later (accounts for row heights)
        let mut form_visual_offset: Option<u16> = None;

        let rows: Vec<Row> = {
            let mut rows = Vec::new();

            // Check if form is active
            if state.input_mode == InputMode::TransactionForm {
                if let Some(ref form_state) = state.form_state {
                    let subtransaction_count = if form_state.is_split_mode {
                        form_state.subtransactions.len() + 1 // +1 for hint row
                    } else {
                        0
                    };

                    if let Some(ref edit_id) = form_state.editing_transaction_id {
                        // EDIT MODE: Replace the transaction being edited with placeholder
                        // Track cumulative visual height for correct Y positioning
                        let mut visual_offset: u16 = 0;
                        for transaction in filtered.iter() {
                            if transaction.id.to_string() == *edit_id {
                                form_visual_offset = Some(visual_offset);
                                // Add placeholder row (will be rendered directly)
                                rows.push(Row::new(vec![Cell::from(""); 8]));
                                // Add placeholder rows for subtransactions
                                for _ in 0..subtransaction_count {
                                    rows.push(Row::new(vec![Cell::from(""); 8]));
                                }
                                visual_offset += 1 + subtransaction_count as u16;
                            } else {
                                let row_height = calculate_row_height(transaction);
                                rows.push(build_transaction_row(transaction, budget));
                                visual_offset += row_height;
                            }
                        }
                    } else {
                        // CREATE MODE: Insert placeholder at top
                        form_visual_offset = Some(0);
                        rows.push(Row::new(vec![Cell::from(""); 8]));
                        // Add placeholder rows for subtransactions
                        for _ in 0..subtransaction_count {
                            rows.push(Row::new(vec![Cell::from(""); 8]));
                        }
                        // Then add all existing transactions
                        for transaction in filtered.iter() {
                            rows.push(build_transaction_row(transaction, budget));
                        }
                    }
                } else {
                    rows = filtered
                        .iter()
                        .map(|t| build_transaction_row(t, budget))
                        .collect();
                }
            } else {
                // Normal rendering without form
                rows = filtered
                    .iter()
                    .map(|t| build_transaction_row(t, budget))
                    .collect();
            }

            rows
        };

        // Update table title to show filter status
        let title = if !state.filter_query.is_empty() {
            format!("Transactions ({} filtered)", filtered.len())
        } else {
            "Transactions".to_string()
        };

        let mut table = Table::new(
            rows,
            [
                Constraint::Length(1),
                Constraint::Length(10),
                Constraint::Percentage(30),
                Constraint::Percentage(25),
                Constraint::Percentage(30),
                Constraint::Percentage(15),
                Constraint::Length(1),
                Constraint::Length(1),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .column_spacing(theme::TABLE_COLUMN_SPACING);

        if !matches!(state.input_mode, InputMode::TransactionForm) {
            table = table.row_highlight_style(theme::selection_style());
        }

        f.render_stateful_widget(table, table_area, &mut state.table_state.borrow_mut());

        // Render form row directly if active (for precise autocomplete positioning)
        if let (Some(visual_offset), Some(ref form_state)) = (form_visual_offset, &state.form_state)
        {
            let table_inner = Block::default().borders(Borders::ALL).inner(table_area);
            let header_height = 1u16;

            // Calculate the Y position of the form row using visual offset
            // (accounts for row heights of transactions with subtransactions)
            let form_y = table_inner.y + header_height + visual_offset;
            let form_row_area = Rect::new(table_inner.x, form_y, table_inner.width, 1);

            // Render the form row directly (returns payee and category areas for dropdown)
            let (payee_area, category_area) = inline_transaction_form::render_form_row_direct(
                f,
                form_row_area,
                form_state,
                budget,
            );

            // Render subtransaction rows if in split mode
            if form_state.is_split_mode && !form_state.subtransactions.is_empty() {
                let subtrans_start = Rect::new(table_inner.x, form_y + 1, table_inner.width, 1);
                inline_transaction_form::render_subtransaction_rows_direct(
                    f,
                    subtrans_start,
                    form_state,
                );
            }

            // Render autocomplete dropdowns last (on top of everything)
            inline_transaction_form::render_form_dropdowns(
                f,
                payee_area,
                category_area,
                form_state,
            );
        }
    } else {
        // No matching transactions - show message based on filter state
        let message = if !state.filter_query.is_empty() {
            "No matching transactions"
        } else {
            "No transactions found"
        };

        empty_state::render_empty_state(f, table_area, "Transactions", message, None);
    }
}

fn calculate_row_height(transaction: &Transaction) -> u16 {
    1 + transaction
        .subtransactions
        .iter()
        .filter(|sub| !sub.deleted)
        .count() as u16
}

fn build_parent_line(
    transaction: &Transaction,
    column: &str,
    budget: Option<&BudgetSummary>,
) -> Line<'static> {
    match column {
        "date" => {
            let date_iso = transaction.date.format("%Y-%m-%d").to_string();
            let date_str = if let Some(budget) = budget {
                if let Some(ref date_format) = budget.date_format {
                    utils::fmt_date(&date_iso, date_format)
                } else {
                    date_iso
                }
            } else {
                date_iso
            };
            Line::from(date_str)
        }
        "payee" => Line::from(transaction.payee_name.as_deref().unwrap_or("-").to_string()),
        "category" => {
            let active_subs = transaction
                .subtransactions
                .iter()
                .filter(|s| !s.deleted)
                .count();
            if active_subs > 0 {
                Line::from("Split (Multiple Categories)...")
            } else {
                Line::from(
                    transaction
                        .category_name
                        .as_deref()
                        .unwrap_or("-")
                        .to_string(),
                )
            }
        }
        "memo" => Line::from(transaction.memo.as_deref().unwrap_or("").to_string()),
        "amount" => {
            let amount_str = utils::format_amount(transaction.amount.into(), budget);
            let amount_color = utils::get_amount_color(transaction.amount.into());
            Line::from(Span::from(amount_str).style(Style::default().fg(amount_color)))
        }
        "flag" => match &transaction.flag_color {
            Some(color) => {
                let ratatui_color = utils::flag_color_to_ratatui_color(color);
                Line::from(Span::from("▰").style(Style::default().fg(ratatui_color)))
            }
            None => Line::from("▱"),
        },
        "approved" => {
            if transaction.approved {
                Line::from(" ")
            } else if transaction.matched_transaction_id.is_some() {
                Line::from(Span::styled("⛓", Style::default().fg(theme::COLOR_TITLE)))
            } else {
                Line::from("ⓘ")
            }
        }
        "cleared" => match transaction.cleared {
            ReconciliationStatus::Uncleared => Line::from("U"),
            ReconciliationStatus::Cleared => {
                Line::from(Span::from("C").style(Style::default().fg(theme::COLOR_POSITIVE)))
            }
            ReconciliationStatus::Reconciled => {
                Line::from(Span::from("R").style(Style::default().fg(Color::Indexed(240))))
            }
        },
        _ => Line::from(""),
    }
}

fn build_subtransaction_line(
    parent_transaction: &Transaction,
    subtransaction: &SubTransaction,
    column: &str,
    budget: Option<&BudgetSummary>,
) -> Line<'static> {
    match column {
        "date" => Line::from(""),
        "payee" => {
            // Only show if different from parent
            let sub_payee = subtransaction
                .payee_name
                .clone()
                .unwrap_or_else(|| String::from("-"));
            let parent_payee = parent_transaction.payee_name.as_deref().unwrap_or("-");

            if sub_payee != parent_payee && sub_payee != "-" {
                Line::from(vec![Span::from("  └─ "), Span::from(sub_payee)])
            } else {
                Line::from("")
            }
        }
        "category" => {
            let category = subtransaction
                .category_name
                .clone()
                .unwrap_or_else(|| String::from("-"));
            Line::from(vec![Span::from("  └─ "), Span::from(category)])
        }
        "memo" => {
            if let Some(ref memo) = subtransaction.memo {
                if !memo.is_empty() {
                    Line::from(memo.clone())
                } else {
                    Line::from("")
                }
            } else {
                Line::from("")
            }
        }
        "amount" => {
            let amount_str = utils::format_amount(subtransaction.amount.into(), budget);
            let amount_color = utils::get_amount_color(subtransaction.amount.into());
            Line::from(Span::from(amount_str).style(Style::default().fg(amount_color)))
        }
        "flag" | "approved" | "cleared" => Line::from(""),
        _ => Line::from(""),
    }
}

fn build_multiline_cell(
    transaction: &Transaction,
    column: &str,
    budget: Option<&BudgetSummary>,
) -> Text<'static> {
    let mut lines = vec![build_parent_line(transaction, column, budget)];

    for subtransaction in transaction
        .subtransactions
        .iter()
        .filter(|sub| !sub.deleted)
        .sorted()
    {
        lines.push(build_subtransaction_line(
            transaction,
            subtransaction,
            column,
            budget,
        ));
    }

    Text::from(lines)
}

fn build_transaction_row(
    transaction: &Transaction,
    budget: Option<&BudgetSummary>,
) -> Row<'static> {
    let row_height = calculate_row_height(transaction);

    // Build multi-line content for each column
    let flag_cell = build_multiline_cell(transaction, "flag", budget);
    let date_cell = build_multiline_cell(transaction, "date", budget);
    let payee_cell = build_multiline_cell(transaction, "payee", budget);
    let category_cell = build_multiline_cell(transaction, "category", budget);
    let memo_cell = build_multiline_cell(transaction, "memo", budget);
    let amount_cell = build_multiline_cell(transaction, "amount", budget);
    let approved_cell = build_multiline_cell(transaction, "approved", budget);
    let cleared_cell = build_multiline_cell(transaction, "cleared", budget);

    // Row styling (bold if unapproved)
    let row_style = if transaction.approved {
        Style::default()
    } else {
        Style::default().bold()
    };

    Row::new(vec![
        Cell::from(flag_cell),
        Cell::from(date_cell),
        Cell::from(payee_cell),
        Cell::from(category_cell),
        Cell::from(memo_cell),
        Cell::from(amount_cell.right_aligned()),
        Cell::from(approved_cell),
        Cell::from(cleared_cell),
    ])
    .style(row_style)
    .height(row_height)
}

/// Render the balance summary showing cleared, uncleared, and working balances as cards
fn render_balance_summary(
    f: &mut Frame,
    area: Rect,
    state: &TransactionsState,
    budget: Option<&BudgetSummary>,
) {
    // Calculate balances from transactions
    let (cleared_balance, uncleared_balance) =
        state
            .transactions
            .iter()
            .fold((0i64, 0i64), |(cleared, uncleared), t| {
                let amount: i64 = t.amount.into();
                match t.cleared {
                    ReconciliationStatus::Cleared | ReconciliationStatus::Reconciled => {
                        (cleared + amount, uncleared)
                    }
                    ReconciliationStatus::Uncleared => (cleared, uncleared + amount),
                }
            });
    let working_balance = cleared_balance + uncleared_balance;

    // Split into cards with operators between them
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Cleared card
            Constraint::Length(3),      // +
            Constraint::Percentage(30), // Uncleared card
            Constraint::Length(3),      // =
            Constraint::Percentage(30), // Working card
        ])
        .split(area);

    // Card 1: Cleared Balance
    let cleared_str = utils::format_amount(cleared_balance, budget);
    render_balance_card(
        f,
        chunks[0],
        &cleared_str,
        "Cleared",
        utils::get_amount_color(cleared_balance),
    );

    // Plus sign (vertically centered on 3-unit card: top border, content, bottom border)
    let plus_area = Rect {
        x: chunks[1].x,
        y: chunks[1].y + 1,
        width: chunks[1].width,
        height: 1,
    };
    let plus = Paragraph::new("+")
        .style(Style::default().fg(theme::COLOR_ZERO))
        .alignment(Alignment::Center);
    f.render_widget(plus, plus_area);

    // Card 2: Uncleared Balance
    let uncleared_str = utils::format_amount(uncleared_balance, budget);
    render_balance_card(
        f,
        chunks[2],
        &uncleared_str,
        "Uncleared",
        theme::COLOR_HELP_TEXT,
    );

    // Equals sign (vertically centered on 3-unit card)
    let equals_area = Rect {
        x: chunks[3].x,
        y: chunks[3].y + 1,
        width: chunks[3].width,
        height: 1,
    };
    let equals = Paragraph::new("=")
        .style(Style::default().fg(theme::COLOR_ZERO))
        .alignment(Alignment::Center);
    f.render_widget(equals, equals_area);

    // Card 3: Working Balance
    let working_str = utils::format_amount(working_balance, budget);
    render_balance_card(
        f,
        chunks[4],
        &working_str,
        "Working",
        utils::get_amount_color(working_balance),
    );
}

fn render_balance_card(f: &mut Frame, area: Rect, amount: &str, label: &str, color: Color) {
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
