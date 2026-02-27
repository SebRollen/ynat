use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Cell, Row},
    Frame,
};

use crate::state::{FormField, SubTransactionField, TransactionFormState};
use crate::ui::{components::autocomplete_input::AutocompleteInput, theme, utils};
use ynab_api::endpoints::budgets::BudgetSummary;

/// Which field the autocomplete dropdown is anchored to
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutocompleteField {
    Payee,
    Category,
    SubtransactionCategory { index: usize },
}

/// Data for rendering the autocomplete dropdown overlay
#[derive(Debug, Clone)]
pub struct AutocompleteOverlay {
    pub field: AutocompleteField,
    pub items: Vec<String>,
    pub selected_index: usize,
    pub hint: Option<&'static str>,
}

/// Get autocomplete overlay info if one should be displayed
pub fn get_autocomplete_overlay(form_state: &TransactionFormState) -> Option<AutocompleteOverlay> {
    match form_state.current_field {
        Some(FormField::Payee) if !form_state.filtered_payees.is_empty() => {
            Some(AutocompleteOverlay {
                field: AutocompleteField::Payee,
                items: form_state
                    .filtered_payees
                    .iter()
                    .take(10)
                    .map(|p| p.name.clone())
                    .collect(),
                selected_index: form_state.payee_selection_index,
                hint: None,
            })
        }
        Some(FormField::Category) if form_state.is_split_mode => {
            // Show hint to exit split mode
            Some(AutocompleteOverlay {
                field: AutocompleteField::Category,
                items: vec![],
                selected_index: 0,
                hint: Some("Type to exit split mode"),
            })
        }
        Some(FormField::Category) if !form_state.filtered_categories.is_empty() => {
            Some(AutocompleteOverlay {
                field: AutocompleteField::Category,
                items: form_state
                    .filtered_categories
                    .iter()
                    .take(10)
                    .map(|c| c.name.clone())
                    .collect(),
                selected_index: form_state.category_selection_index,
                hint: Some("Ctrl+S to split"),
            })
        }
        _ => {
            // Check subtransaction category autocomplete
            if let Some(active_index) = form_state.active_subtransaction_index {
                if form_state.subtransaction_field == SubTransactionField::Category {
                    if let Some(sub) = form_state.subtransactions.get(active_index) {
                        if !sub.filtered_categories.is_empty() {
                            return Some(AutocompleteOverlay {
                                field: AutocompleteField::SubtransactionCategory {
                                    index: active_index,
                                },
                                items: sub
                                    .filtered_categories
                                    .iter()
                                    .take(10)
                                    .map(|c| c.name.clone())
                                    .collect(),
                                selected_index: sub.category_selection_index,
                                hint: None,
                            });
                        }
                    }
                }
            }
            None
        }
    }
}

/// Column constraints matching the transaction table layout
pub const FORM_COLUMN_CONSTRAINTS: [Constraint; 8] = [
    Constraint::Length(1),      // Flag
    Constraint::Length(10),     // Date
    Constraint::Percentage(30), // Payee
    Constraint::Percentage(25), // Category
    Constraint::Percentage(30), // Memo
    Constraint::Percentage(15), // Amount
    Constraint::Length(1),      // Approved
    Constraint::Length(1),      // Cleared
];

/// Check if there's a validation error to display
pub fn has_validation_error(form_state: &TransactionFormState) -> bool {
    form_state.validation_error.is_some()
}

/// Render the validation error message above the table
pub fn render_validation_error(f: &mut Frame, area: Rect, form_state: &TransactionFormState) {
    if let Some(ref error) = form_state.validation_error {
        let error_text = format!(" Error: {}", error);
        let paragraph = ratatui::widgets::Paragraph::new(
            Span::from(error_text).style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .style(Style::default().bg(theme::COLOR_NEGATIVE).fg(Color::White));
        f.render_widget(paragraph, area);
    }
}

/// Render the form row directly to the frame at the given area.
/// This gives us precise control over field positions for autocomplete dropdowns.
/// Returns the areas for payee and category fields (for deferred dropdown rendering).
pub fn render_form_row_direct(
    f: &mut Frame,
    row_area: Rect,
    form_state: &TransactionFormState,
    budget: Option<&BudgetSummary>,
) -> (Rect, Rect) {
    // Split the row area into columns matching the table layout
    let col_spacing = theme::TABLE_COLUMN_SPACING;
    let columns = Layout::horizontal(FORM_COLUMN_CONSTRAINTS)
        .spacing(col_spacing)
        .split(row_area);

    // Render each field in its column (without autocomplete dropdowns)
    render_flag_field(f, columns[0], form_state);
    render_date_field(f, columns[1], form_state, budget);
    render_payee_field_no_dropdown(f, columns[2], form_state);
    render_category_field_no_dropdown(f, columns[3], form_state);
    render_memo_field(f, columns[4], form_state);
    render_amount_field(f, columns[5], form_state);
    // columns[6] is approved (empty for form)
    render_cleared_field(f, columns[7], form_state);

    // Return areas for deferred dropdown rendering
    (columns[2], columns[3])
}

/// Render autocomplete dropdowns for the form row (call after subtransaction rows)
pub fn render_form_dropdowns(
    f: &mut Frame,
    payee_area: Rect,
    category_area: Rect,
    form_state: &TransactionFormState,
) {
    render_payee_dropdown(f, payee_area, form_state);
    render_category_dropdown(f, category_area, form_state);
}

/// Render subtransaction rows directly
pub fn render_subtransaction_rows_direct(
    f: &mut Frame,
    start_area: Rect,
    form_state: &TransactionFormState,
) {
    let col_spacing = theme::TABLE_COLUMN_SPACING;

    for (index, sub) in form_state.subtransactions.iter().enumerate() {
        let row_y = start_area.y + index as u16;
        if row_y >= f.area().height {
            break;
        }

        let row_area = Rect::new(start_area.x, row_y, start_area.width, 1);
        let columns = Layout::horizontal(FORM_COLUMN_CONSTRAINTS)
            .spacing(col_spacing)
            .split(row_area);

        let is_active = form_state.active_subtransaction_index == Some(index);

        // Render prefix in date column
        let prefix = format!("  └─ #{}", index + 1);
        let prefix_span = Span::from(prefix).style(Style::default().fg(Color::DarkGray));
        f.render_widget(prefix_span, columns[1]);

        // Render subtransaction category field
        render_subtransaction_category_field(f, columns[3], form_state, sub, index, is_active);

        // Render subtransaction memo field
        render_subtransaction_memo_field(f, columns[4], form_state, sub, is_active);

        // Render subtransaction amount field
        render_subtransaction_amount_field(f, columns[5], form_state, sub, is_active);
    }

    // Render hint row after all subtransactions
    let hint_row_y = start_area.y + form_state.subtransactions.len() as u16;
    if hint_row_y < f.area().height {
        let hint_row_area = Rect::new(start_area.x, hint_row_y, start_area.width, 1);
        let columns = Layout::horizontal(FORM_COLUMN_CONSTRAINTS)
            .spacing(col_spacing)
            .split(hint_row_area);

        // Render keyboard hints in payee column
        let hint_text = "[Ctrl+A] Add split  [Ctrl+D] Delete";
        let hint_span = Span::from(hint_text).style(Style::default().fg(Color::DarkGray));
        f.render_widget(hint_span, columns[2]);

        // Calculate and render remaining amount in amount column
        let parent_amount: f64 = form_state.amount.parse().unwrap_or(0.0);
        let sum_of_splits: f64 = form_state
            .subtransactions
            .iter()
            .filter_map(|s| s.amount.parse::<f64>().ok())
            .sum();
        let remaining = parent_amount - sum_of_splits;

        let (remaining_text, remaining_style) = if remaining.abs() < 0.001 {
            (
                "✓ Balanced".to_string(),
                Style::default().fg(theme::COLOR_POSITIVE),
            )
        } else {
            (
                format!("{:+.2} remaining", remaining),
                Style::default().fg(theme::COLOR_NEGATIVE),
            )
        };

        let remaining_span = Span::from(remaining_text).style(remaining_style);
        // Right-align the remaining text in the amount column
        let text_width = remaining_span.width() as u16;
        let amount_col = columns[5];
        let right_aligned_x = if amount_col.width > text_width {
            amount_col.x + amount_col.width - text_width
        } else {
            amount_col.x
        };
        let aligned_area = Rect::new(right_aligned_x, amount_col.y, text_width, 1);
        f.render_widget(remaining_span, aligned_area);
    }
}

fn render_flag_field(f: &mut Frame, area: Rect, form_state: &TransactionFormState) {
    let is_focused = form_state.current_field == Some(FormField::FlagColor);
    let widget = match &form_state.flag_color {
        Some(color) => {
            let ratatui_color = utils::flag_color_to_ratatui_color(color);
            let style = if is_focused {
                theme::form_field_focused_style().fg(ratatui_color)
            } else {
                Style::default().fg(ratatui_color)
            };
            Span::from("▰").style(style)
        }
        None => {
            let style = if is_focused {
                theme::form_field_focused_style()
            } else {
                Style::default()
            };
            Span::from("▱").style(style)
        }
    };
    f.render_widget(widget, area);
}

fn render_date_field(
    f: &mut Frame,
    area: Rect,
    form_state: &TransactionFormState,
    budget: Option<&BudgetSummary>,
) {
    let is_focused = form_state.current_field == Some(FormField::Date);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = if form_state.date.is_empty() {
        budget
            .and_then(|b| b.date_format.clone().map(|d| d.format))
            .unwrap_or_else(|| "YYYY-MM-DD".to_string())
    } else {
        form_state.date.clone()
    };

    f.render_widget(Span::from(value).style(style), area);
}

/// Render payee field without the autocomplete dropdown (just the input text)
fn render_payee_field_no_dropdown(f: &mut Frame, area: Rect, form_state: &TransactionFormState) {
    let is_focused = form_state.current_field == Some(FormField::Payee);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = if form_state.payee.is_empty() {
        "_____________"
    } else {
        &form_state.payee
    };

    f.render_widget(Span::from(value).style(style), area);
}

/// Render the payee autocomplete dropdown (if applicable)
fn render_payee_dropdown(f: &mut Frame, area: Rect, form_state: &TransactionFormState) {
    let is_focused = form_state.current_field == Some(FormField::Payee);
    let has_autocomplete = is_focused && !form_state.filtered_payees.is_empty();

    if has_autocomplete {
        let items: Vec<String> = form_state
            .filtered_payees
            .iter()
            .take(10)
            .map(|p| p.name.clone())
            .collect();

        AutocompleteInput::new(&form_state.payee, "_____________")
            .focused(true)
            .items(&items)
            .selected_index(form_state.payee_selection_index)
            .render(f, area);
    }
}

/// Render category field without the autocomplete dropdown (just the input text)
fn render_category_field_no_dropdown(f: &mut Frame, area: Rect, form_state: &TransactionFormState) {
    let is_focused = form_state.current_field == Some(FormField::Category);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    // In split mode, show "Split (N)"
    if form_state.is_split_mode {
        let value = format!("Split ({})", form_state.subtransactions.len());
        f.render_widget(Span::from(value).style(style), area);
        return;
    }

    let value = if form_state.category.is_empty() {
        "_____________"
    } else {
        &form_state.category
    };

    f.render_widget(Span::from(value).style(style), area);
}

/// Render the category autocomplete dropdown (if applicable)
fn render_category_dropdown(f: &mut Frame, area: Rect, form_state: &TransactionFormState) {
    let is_focused = form_state.current_field == Some(FormField::Category);

    // In split mode, show hint dropdown when focused
    if form_state.is_split_mode {
        if is_focused {
            let value = format!("Split ({})", form_state.subtransactions.len());
            AutocompleteInput::new(&value, "")
                .focused(true)
                .hint(Some("Type to exit split mode"))
                .render(f, area);
        }
        return;
    }

    let has_autocomplete = is_focused && !form_state.filtered_categories.is_empty();

    if has_autocomplete {
        let items: Vec<String> = form_state
            .filtered_categories
            .iter()
            .take(10)
            .map(|c| c.name.clone())
            .collect();

        AutocompleteInput::new(&form_state.category, "_____________")
            .focused(true)
            .items(&items)
            .selected_index(form_state.category_selection_index)
            .hint(Some("Ctrl+S to split"))
            .render(f, area);
    }
}

fn render_memo_field(f: &mut Frame, area: Rect, form_state: &TransactionFormState) {
    let is_focused = form_state.current_field == Some(FormField::Memo);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = if form_state.memo.is_empty() {
        "_____________"
    } else {
        &form_state.memo
    };

    f.render_widget(Span::from(value).style(style), area);
}

fn render_amount_field(f: &mut Frame, area: Rect, form_state: &TransactionFormState) {
    let is_focused = form_state.current_field == Some(FormField::Amount);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = if form_state.amount.is_empty() {
        "_______".to_string()
    } else {
        form_state.amount.clone()
    };

    // Right-align the amount
    let text = Text::from(Line::from(Span::from(value).style(style))).right_aligned();
    f.render_widget(text, area);
}

fn render_cleared_field(f: &mut Frame, area: Rect, form_state: &TransactionFormState) {
    let is_focused = form_state.current_field == Some(FormField::Cleared);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = format!("{}", form_state.cleared);
    f.render_widget(Span::from(value).style(style), area);
}

fn render_subtransaction_category_field(
    f: &mut Frame,
    area: Rect,
    form_state: &TransactionFormState,
    sub: &crate::state::SubTransactionFormState,
    _index: usize,
    is_active: bool,
) {
    let is_focused = is_active && form_state.subtransaction_field == SubTransactionField::Category;
    let has_autocomplete = is_focused && !sub.filtered_categories.is_empty();

    let value = if sub.category.is_empty() {
        "_____________"
    } else {
        &sub.category
    };

    if has_autocomplete {
        let items: Vec<String> = sub
            .filtered_categories
            .iter()
            .take(10)
            .map(|c| c.name.clone())
            .collect();

        AutocompleteInput::new(value, "_____________")
            .focused(true)
            .items(&items)
            .selected_index(sub.category_selection_index)
            .render(f, area);
    } else {
        let style = if is_focused {
            theme::form_field_focused_style()
        } else if is_active {
            theme::form_field_style()
        } else {
            Style::default().fg(Color::DarkGray)
        };
        f.render_widget(Span::from(value).style(style), area);
    }
}

fn render_subtransaction_memo_field(
    f: &mut Frame,
    area: Rect,
    form_state: &TransactionFormState,
    sub: &crate::state::SubTransactionFormState,
    is_active: bool,
) {
    let is_focused = is_active && form_state.subtransaction_field == SubTransactionField::Memo;
    let style = if is_focused {
        theme::form_field_focused_style()
    } else if is_active {
        theme::form_field_style()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let value = if sub.memo.is_empty() {
        "_______"
    } else {
        &sub.memo
    };

    f.render_widget(Span::from(value).style(style), area);
}

fn render_subtransaction_amount_field(
    f: &mut Frame,
    area: Rect,
    form_state: &TransactionFormState,
    sub: &crate::state::SubTransactionFormState,
    is_active: bool,
) {
    let is_focused = is_active && form_state.subtransaction_field == SubTransactionField::Amount;
    let style = if is_focused {
        theme::form_field_focused_style()
    } else if is_active {
        theme::form_field_style()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let value = if sub.amount.is_empty() {
        "______".to_string()
    } else {
        sub.amount.clone()
    };

    let text = Text::from(Line::from(Span::from(value).style(style))).right_aligned();
    f.render_widget(text, area);
}

/// Main entry point - returns Vec<Row> (form row + optional error row + subtransaction rows)
pub fn render_inline_transaction_form(
    form_state: &TransactionFormState,
    budget: Option<&BudgetSummary>,
) -> Vec<Row<'static>> {
    let mut rows = Vec::new();

    // Calculate row height based on autocomplete state
    let row_height = calculate_form_row_height(form_state);

    // Build cells for each column
    let flag_cell = build_flag_cell(form_state);
    let date_cell = build_date_cell(
        form_state,
        budget.and_then(|b| b.date_format.clone().map(|d| d.format)),
    );
    let payee_cell = build_payee_cell(form_state);
    let category_cell = build_category_cell(form_state);
    let memo_cell = build_memo_cell(form_state);
    let amount_text = build_amount_text(form_state);
    let approved_cell = Cell::from("");
    let cleared_cell = build_cleared_cell(form_state);

    // Create the form row
    let form_row = Row::new(vec![
        flag_cell,
        date_cell,
        payee_cell,
        category_cell,
        memo_cell,
        Cell::from(amount_text.right_aligned()),
        approved_cell,
        cleared_cell,
    ])
    .height(row_height);

    rows.push(form_row);

    // Add subtransaction rows if in split mode
    if form_state.is_split_mode {
        rows.extend(build_subtransaction_rows(form_state));
    }

    // Add error row if validation error exists
    if let Some(ref error) = form_state.validation_error {
        rows.push(build_error_row(error));
    }

    rows
}

/// Calculate dynamic row height - now always returns 1 since autocomplete is rendered as overlay
fn calculate_form_row_height(_form_state: &TransactionFormState) -> u16 {
    1
}

/// Build date cell with focus indicator
fn build_date_cell(
    form_state: &TransactionFormState,
    date_format: Option<String>,
) -> Cell<'static> {
    let is_focused = form_state.current_field == Some(FormField::Date);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = if form_state.date.is_empty() {
        date_format.unwrap_or("YYYY-MM-DD".to_string())
    } else {
        form_state.date.clone()
    };

    Cell::from(value).style(style)
}

/// Build amount text with focus indicator
fn build_amount_text(form_state: &TransactionFormState) -> Text<'static> {
    let is_focused = form_state.current_field == Some(FormField::Amount);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = if form_state.amount.is_empty() {
        "_______".to_string()
    } else {
        form_state.amount.clone()
    };

    Text::from(Line::from(Span::from(value).style(style)))
}

fn build_flag_cell(form_state: &TransactionFormState) -> Cell<'static> {
    let is_focused = form_state.current_field == Some(FormField::FlagColor);
    let line = match &form_state.flag_color {
        Some(color) => {
            let ratatui_color = utils::flag_color_to_ratatui_color(color);
            let style = if is_focused {
                theme::form_field_focused_style().fg(ratatui_color)
            } else {
                Style::default().fg(ratatui_color)
            };
            Line::from(Span::from("▰").style(style))
        }
        None => {
            let style = if is_focused {
                theme::form_field_focused_style()
            } else {
                Style::default()
            };
            Line::from("▱").style(style)
        }
    };
    Cell::from(line)
}

/// Build payee cell with focus indicator (autocomplete rendered as overlay)
fn build_payee_cell(form_state: &TransactionFormState) -> Cell<'static> {
    let is_focused = form_state.current_field == Some(FormField::Payee);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = if form_state.payee.is_empty() {
        "_____________".to_string()
    } else {
        form_state.payee.clone()
    };

    Cell::from(Span::from(value).style(style))
}

/// Build category cell with focus indicator (autocomplete rendered as overlay)
fn build_category_cell(form_state: &TransactionFormState) -> Cell<'static> {
    let is_focused = form_state.current_field == Some(FormField::Category);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    // In split mode, show "Split (N)"
    if form_state.is_split_mode {
        let split_count = form_state.subtransactions.len();
        let value = format!("Split ({})", split_count);
        return Cell::from(Span::from(value).style(style));
    }

    let value = if form_state.category.is_empty() {
        "_____________".to_string()
    } else {
        form_state.category.clone()
    };

    Cell::from(Span::from(value).style(style))
}

/// Build memo cell with focus indicator
fn build_memo_cell(form_state: &TransactionFormState) -> Cell<'static> {
    let is_focused = form_state.current_field == Some(FormField::Memo);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = if form_state.memo.is_empty() {
        "_____________".to_string()
    } else {
        form_state.memo.clone()
    };

    Cell::from(Text::from(Line::from(Span::from(value).style(style))))
}

/// Build cleared cell with focus indicator and hint
fn build_cleared_cell(form_state: &TransactionFormState) -> Cell<'static> {
    let is_focused = form_state.current_field == Some(FormField::Cleared);
    let style = if is_focused {
        theme::form_field_focused_style()
    } else {
        theme::form_field_style()
    };

    let value = format!("{}", form_state.cleared);

    Cell::from(Text::from(Line::from(Span::from(value).style(style))))
}

/// Build error row to display validation errors
fn build_error_row(error: &str) -> Row<'static> {
    let error_text = format!(" Error: {}", error);
    let cell = Cell::from(Text::from(
        Span::from(error_text).style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ));

    // Create a row with the error spanning all columns
    Row::new(vec![
        Cell::from(""),
        Cell::from(""),
        cell,
        Cell::from(""),
        Cell::from(""),
        Cell::from(""),
        Cell::from(""),
        Cell::from(""),
    ])
    .style(Style::default().bg(theme::COLOR_NEGATIVE).fg(Color::White))
}

/// Build rows for subtransactions in split mode
fn build_subtransaction_rows(form_state: &TransactionFormState) -> Vec<Row<'static>> {
    let mut rows = Vec::new();

    for (index, sub) in form_state.subtransactions.iter().enumerate() {
        let is_active = form_state.active_subtransaction_index == Some(index);
        let row_height = calculate_subtransaction_row_height(form_state, index, is_active);

        // Build the subtransaction row
        let row = build_subtransaction_row(form_state, sub, index, is_active);
        rows.push(row.height(row_height));
    }

    // Add hint row for split mode controls
    if !form_state.subtransactions.is_empty() {
        rows.push(build_split_mode_hint_row(form_state));
    }

    rows
}

/// Calculate row height for a subtransaction - now always 1 since autocomplete is overlay
fn calculate_subtransaction_row_height(
    _form_state: &TransactionFormState,
    _index: usize,
    _is_active: bool,
) -> u16 {
    1
}

/// Build a single subtransaction row
fn build_subtransaction_row(
    form_state: &TransactionFormState,
    sub: &crate::state::SubTransactionFormState,
    index: usize,
    is_active: bool,
) -> Row<'static> {
    // Prefix indicator showing this is a split item
    let prefix = format!("  └─ #{}", index + 1);

    // Amount field
    let amount_focused =
        is_active && form_state.subtransaction_field == SubTransactionField::Amount;
    let amount_style = if amount_focused {
        theme::form_field_focused_style()
    } else if is_active {
        theme::form_field_style()
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let amount_value = if sub.amount.is_empty() {
        "______".to_string()
    } else {
        sub.amount.clone()
    };

    // Category field (autocomplete rendered as overlay)
    let category_focused =
        is_active && form_state.subtransaction_field == SubTransactionField::Category;
    let category_style = if category_focused {
        theme::form_field_focused_style()
    } else if is_active {
        theme::form_field_style()
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let category_value = if sub.category.is_empty() {
        "_____________".to_string()
    } else {
        sub.category.clone()
    };

    // Memo field
    let memo_focused = is_active && form_state.subtransaction_field == SubTransactionField::Memo;
    let memo_style = if memo_focused {
        theme::form_field_focused_style()
    } else if is_active {
        theme::form_field_style()
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let memo_value = if sub.memo.is_empty() {
        "_______".to_string()
    } else {
        sub.memo.clone()
    };

    // Build the row: prefix spans first 3 columns, then category, memo, amount
    Row::new(vec![
        Cell::from(""), // Flag column (empty for subtransactions)
        Cell::from(Text::from(
            Span::from(prefix).style(Style::default().fg(Color::DarkGray)),
        )), // Date column shows prefix
        Cell::from(""), // Payee column (empty for subtransactions)
        Cell::from(Span::from(category_value).style(category_style)), // Category
        Cell::from(Span::from(memo_value).style(memo_style)), // Memo
        Cell::from(Text::from(Span::from(amount_value).style(amount_style)).right_aligned()), // Amount
        Cell::from(""), // Approved column
        Cell::from(""), // Cleared column
    ])
}

/// Build hint row showing split mode keyboard shortcuts
fn build_split_mode_hint_row(form_state: &TransactionFormState) -> Row<'static> {
    // Calculate remaining amount
    let parent_amount: f64 = form_state.amount.parse().unwrap_or(0.0);
    let sum_of_splits: f64 = form_state
        .subtransactions
        .iter()
        .filter_map(|s| s.amount.parse::<f64>().ok())
        .sum();
    let remaining = parent_amount - sum_of_splits;

    let remaining_text = if remaining.abs() < 0.001 {
        "✓ Balanced".to_string()
    } else {
        format!("{:+.2} remaining", remaining)
    };

    let remaining_style = if remaining.abs() < 0.001 {
        Style::default().fg(theme::COLOR_POSITIVE)
    } else {
        Style::default().fg(theme::COLOR_NEGATIVE)
    };

    let hint_text = "[Ctrl+N] Add split  [Ctrl+D] Delete";

    Row::new(vec![
        Cell::from(""),
        Cell::from(""),
        Cell::from(Text::from(
            Span::from(hint_text).style(Style::default().fg(Color::DarkGray)),
        )),
        Cell::from(""),
        Cell::from(""),
        Cell::from(Text::from(Span::from(remaining_text).style(remaining_style)).right_aligned()),
        Cell::from(""),
        Cell::from(""),
    ])
}
