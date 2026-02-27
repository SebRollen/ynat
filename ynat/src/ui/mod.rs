pub mod components;
pub mod layouts;
pub mod screens;
pub mod theme;
pub mod utils;

use crate::log_buffer::LogBuffer;
use crate::state::{AppState, InputMode};
use ratatui::Frame;
use screens::*;

/// Pure render dispatcher - routes to appropriate screen renderer
/// This function is read-only and never mutates state
pub fn render_app(f: &mut Frame, state: &AppState, log_buffer: &LogBuffer) {
    // Render the current screen
    match state.current_screen() {
        Screen::Budgets(budgets_state) => {
            budgets_screen::render(f, budgets_state);
        }
        Screen::Accounts(accounts_state) => {
            accounts_screen::render(f, accounts_state, state.current_budget.as_ref());
        }
        Screen::Transactions(transactions_state) => {
            transactions_screen::render(f, transactions_state, state.current_budget.as_ref());

            // Render delete confirmation popup if active
            if transactions_state.input_mode == InputMode::DeleteConfirmation {
                if let Some(ref transaction_id) =
                    transactions_state.delete_confirmation_transaction_id
                {
                    // Find the transaction to show details
                    if transactions_state
                        .transactions
                        .iter()
                        .find(|t| t.id.to_string() == *transaction_id)
                        .is_some()
                    {
                        components::delete_confirmation::render_delete_confirmation(f);
                    }
                }
            }

            // Render reconciled edit confirmation popup if active
            if transactions_state.input_mode == InputMode::ReconciledEditConfirmation {
                if let Some(ref transaction_id) = transactions_state.reconciled_edit_transaction_id
                {
                    // Find the transaction to show details
                    if transactions_state
                        .transactions
                        .iter()
                        .find(|t| t.id.to_string() == *transaction_id)
                        .is_some()
                    {
                        components::reconciled_edit_confirmation::render_reconciled_edit_confirmation(
                            f,
                        );
                    }
                }
            }

            // Render reconcile confirmation popup if active
            if transactions_state.input_mode == InputMode::ReconcileConfirmation {
                if let Some(cleared_balance) = transactions_state.reconcile_cleared_balance {
                    let currency_format = state
                        .current_budget
                        .as_ref()
                        .and_then(|b| b.currency_format.as_ref());
                    components::reconcile_confirmation::render_reconcile_confirmation(
                        f,
                        cleared_balance,
                        currency_format,
                    );
                }
            }
        }
        Screen::Plan(plan_state) => {
            screens::plan_screen::render(f, plan_state, state.current_budget.as_ref());
        }
        Screen::Logs(logs_state) => {
            screens::logs_screen::render(f, logs_state, log_buffer);
        }
    }

    // Render help popup on top if visible
    if state.help_visible {
        components::help_popup::render_help_popup(f, state.current_screen());
    }
}
