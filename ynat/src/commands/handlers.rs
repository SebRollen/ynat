use crate::events::AppCommand;
use crate::input::{Key, KeyEvent};
use crate::state::*;
use crate::ui::screens::Screen;
use ynab_api::endpoints::transactions::ReconciliationStatus;

/// Map user input (KeyEvent) to AppCommand based on current UI state
/// Returns None if the key should be ignored
pub fn handle_key_input(event: KeyEvent, state: &AppState) -> Option<AppCommand> {
    let key = event.key;

    // Priority 0: Budget edit mode on Plan screen (highest priority)
    if let Screen::Plan(plan_state) = state.current_screen() {
        if plan_state.input_mode == InputMode::BudgetEdit {
            return handle_budget_edit_keys(event, state);
        }
    }

    // Priority 1: Transaction form mode (highest priority)
    if let Screen::Transactions(trans_state) = state.current_screen() {
        if trans_state.input_mode == InputMode::TransactionForm {
            return handle_transaction_form_keys(event, trans_state);
        }
    }

    // Priority 2: Delete confirmation popup
    if let Screen::Transactions(trans_state) = state.current_screen() {
        if trans_state.input_mode == InputMode::DeleteConfirmation {
            return handle_delete_confirmation_keys(key, trans_state, state);
        }
    }

    // Priority 2.5: Reconciled edit confirmation popup
    if let Screen::Transactions(trans_state) = state.current_screen() {
        if trans_state.input_mode == InputMode::ReconciledEditConfirmation {
            return handle_reconciled_edit_confirmation_keys(key, trans_state);
        }
    }

    // Priority 2.6: Reconcile confirmation popup
    if let Screen::Transactions(trans_state) = state.current_screen() {
        if trans_state.input_mode == InputMode::ReconcileConfirmation {
            return handle_reconcile_confirmation_keys(key, state);
        }
    }

    // Priority 3: Check if we're in filter mode on any screen that supports filtering
    match state.current_screen() {
        Screen::Transactions(trans_state) => {
            if trans_state.input_mode == InputMode::Filter {
                // Filter mode key handling
                return match key {
                    Key::Enter => Some(AppCommand::ExitFilterMode),
                    Key::Backspace => Some(AppCommand::DeleteFilterChar),
                    Key::Char(c) => Some(AppCommand::AppendFilterChar(c)),
                    Key::Esc => Some(AppCommand::ClearFilter),
                    _ => None,
                };
            }
        }
        Screen::Accounts(accounts_state) => {
            if accounts_state.input_mode == InputMode::Filter {
                // Filter mode key handling
                return match key {
                    Key::Enter => Some(AppCommand::ExitFilterMode),
                    Key::Backspace => Some(AppCommand::DeleteFilterChar),
                    Key::Char(c) => Some(AppCommand::AppendFilterChar(c)),
                    Key::Esc => Some(AppCommand::ClearFilter),
                    _ => None,
                };
            }
        }
        _ => {}
    }

    // Priority 4: Check if we're currently showing the help popup
    // This must come before screen-specific Esc handling so help popup takes precedence
    if state.help_visible {
        return match key {
            Key::Char('?') | Key::Esc => Some(AppCommand::ToggleHelp),
            Key::Char('q') => Some(AppCommand::Quit),
            _ => None,
        };
    }

    // Priority 5: Screen-specific Esc handling (clear filter when not in filter mode)
    match state.current_screen() {
        Screen::Transactions(_) => {
            if matches!(key, Key::Esc) {
                return Some(AppCommand::ClearFilter);
            }
        }
        Screen::Accounts(_) => {
            if matches!(key, Key::Esc) {
                return Some(AppCommand::ClearFilter);
            }
        }
        _ => {}
    }

    // Handle multi-key sequences
    if let Some(pending) = state.pending_key {
        // We have a pending key, handle the second key in the sequence
        return match (pending, key) {
            // 'g' followed by 'b' -> go to budgets
            ('g', Key::Char('b')) => Some(AppCommand::LoadBudgets {
                force_refresh: false,
                load_accounts: false,
            }),
            // 'g' followed by 'p' -> go to plan
            ('g', Key::Char('p')) => {
                state
                    .current_budget_id
                    .as_ref()
                    .map(|budget_id| AppCommand::LoadPlan {
                        budget_id: budget_id.clone(),
                        force_refresh: false,
                    })
            }
            // 'g' followed by 'g' -> navigate to top of table
            ('g', Key::Char('g')) => Some(AppCommand::NavigateToTop),
            // 'g' followed by 'l' -> go to logs
            ('g', Key::Char('l')) => Some(AppCommand::NavigateToLogs),
            // Any other key clears the pending key
            _ => Some(AppCommand::ClearPendingKey),
        };
    }

    match (state.current_screen(), key) {
        // Global help toggle
        (_, Key::Char('?')) => Some(AppCommand::ToggleHelp),

        // Global quit command
        (_, Key::Char('q')) => Some(AppCommand::Quit),

        // Multi-key sequence initiator: 'g' sets pending key
        (_, Key::Char('g')) => Some(AppCommand::SetPendingKey('g')),

        // Navigate to top: 'G' (Shift+g)
        (_, Key::Char('G')) => Some(AppCommand::NavigateToBottom),

        // Global back navigation (left/h)
        (_, Key::Left | Key::Char('h')) => Some(AppCommand::NavigateBack),

        // Budgets screen
        (Screen::Budgets(..), Key::Up | Key::Char('k')) => Some(AppCommand::SelectPrevious),
        (Screen::Budgets(..), Key::Down | Key::Char('j')) => Some(AppCommand::SelectNext),
        (Screen::Budgets(budgets_state), Key::Enter | Key::Right | Key::Char('l')) => {
            // Load selected budget's accounts
            if !budgets_state.budgets.is_empty() {
                let budget = budgets_state.budgets[budgets_state.selected_budget_index].clone();
                Some(AppCommand::LoadAccounts {
                    budget_id: budget.id.to_string(),
                    budget: Box::new(Some(budget)),
                    force_refresh: false,
                })
            } else {
                None
            }
        }
        (Screen::Budgets(..), Key::Char('r')) => Some(AppCommand::LoadBudgets {
            force_refresh: true,
            load_accounts: false,
        }),

        // Accounts screen
        (Screen::Accounts(..), Key::Char('/')) => Some(AppCommand::EnterFilterMode),
        (Screen::Accounts(..), Key::Char('.')) => Some(AppCommand::ToggleShowClosedAccounts),
        (Screen::Accounts(..), Key::Up | Key::Char('k')) => Some(AppCommand::SelectPrevious),
        (Screen::Accounts(..), Key::Down | Key::Char('j')) => Some(AppCommand::SelectNext),
        (Screen::Accounts(accounts_state), Key::Enter | Key::Right | Key::Char('l')) => {
            // Load transactions for selected account
            if let Some(budget_id) = &state.current_budget_id {
                let filtered_accounts = accounts_state.filtered_accounts();

                if accounts_state.table_state.borrow().selected().unwrap() < filtered_accounts.len()
                {
                    let account_id = filtered_accounts
                        [accounts_state.table_state.borrow().selected().unwrap()]
                    .id
                    .to_string();
                    Some(AppCommand::LoadTransactions {
                        budget_id: budget_id.clone(),
                        account_id,
                        force_refresh: false,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        (Screen::Accounts(..), Key::Char('r')) => {
            // Force refresh accounts
            state
                .current_budget_id
                .as_ref()
                .map(|budget_id| AppCommand::LoadAccounts {
                    budget_id: budget_id.clone(),
                    budget: Box::new(state.current_budget.clone()),
                    force_refresh: true,
                })
        }

        // Transactions screen
        (Screen::Transactions(..), Key::Char('n')) => Some(AppCommand::EnterTransactionCreateMode),
        (Screen::Transactions(transactions_state), Key::Backspace | Key::Char('d')) => {
            // Delete transaction - only in Normal mode with a valid selection
            if transactions_state.input_mode == InputMode::Normal {
                let selected_idx = transactions_state.table_state.borrow().selected()?;
                let filtered_transactions = transactions_state.filtered_transactions();

                if selected_idx < filtered_transactions.len() {
                    let transaction = filtered_transactions[selected_idx];
                    Some(AppCommand::InitiateTransactionDelete {
                        transaction_id: transaction.id.to_string(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        (Screen::Transactions(transactions_state), Key::Char('a')) => {
            let Some(budget_id) = &state.current_budget_id else {
                return None;
            };
            if transactions_state.input_mode == InputMode::Normal {
                let selected_idx = transactions_state.table_state.borrow().selected()?;
                let filtered_transactions = transactions_state.filtered_transactions();

                if selected_idx < filtered_transactions.len() {
                    let transaction = filtered_transactions[selected_idx];
                    if transaction.approved {
                        return None;
                    }

                    Some(AppCommand::ApproveTransaction {
                        budget_id: budget_id.clone(),
                        transaction_id: transaction.id.to_string(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        (Screen::Transactions(transactions_state), Key::Char('e')) => {
            // Edit transaction - only in Normal mode with a valid selection
            if transactions_state.input_mode == InputMode::Normal {
                let selected_idx = transactions_state.table_state.borrow().selected()?;
                let filtered_transactions = transactions_state.filtered_transactions();

                if selected_idx < filtered_transactions.len() {
                    let transaction = filtered_transactions[selected_idx];
                    Some(AppCommand::InitiateTransactionEdit {
                        transaction_id: transaction.id.to_string(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        (Screen::Transactions(..), Key::Char('/')) => Some(AppCommand::EnterFilterMode),
        (Screen::Transactions(..), Key::Up | Key::Char('k')) => Some(AppCommand::SelectPrevious),
        (Screen::Transactions(..), Key::Down | Key::Char('j')) => Some(AppCommand::SelectNext),
        (Screen::Transactions(transactions_state), Key::Char('c')) => {
            // Toggle cleared status of selected transaction
            if let Some(budget_id) = &state.current_budget_id {
                let selected_idx = transactions_state.table_state.borrow().selected()?;
                let filtered_transactions = transactions_state.filtered_transactions();

                if selected_idx < filtered_transactions.len() {
                    let transaction = filtered_transactions[selected_idx];
                    if transaction.cleared != ReconciliationStatus::Reconciled {
                        Some(AppCommand::ToggleTransactionCleared {
                            transaction_id: transaction.id.to_string(),
                            budget_id: budget_id.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        (Screen::Transactions(..), Key::Char('r')) => {
            // Force refresh transactions
            if let Some(budget_id) = &state.current_budget_id {
                state
                    .current_account_id
                    .as_ref()
                    .map(|account_id| AppCommand::LoadTransactions {
                        budget_id: budget_id.clone(),
                        account_id: account_id.clone(),
                        force_refresh: true,
                    })
            } else {
                None
            }
        }
        (Screen::Transactions(..), Key::Char('.')) => {
            Some(AppCommand::ToggleShowReconciledTransactions)
        }
        (Screen::Transactions(trans_state), Key::Char('R')) => {
            // Initiate reconciliation - calculate cleared balance
            if trans_state.input_mode == InputMode::Normal {
                use ynab_api::endpoints::Milliunits;
                let cleared_balance: Milliunits = trans_state
                    .transactions
                    .iter()
                    .filter(|t| {
                        matches!(
                            t.cleared,
                            ReconciliationStatus::Cleared | ReconciliationStatus::Reconciled
                        )
                    })
                    .map(|t| t.amount)
                    .sum();
                Some(AppCommand::InitiateReconcile {
                    cleared_balance: cleared_balance.into(),
                })
            } else {
                None
            }
        }

        // Plan screen
        (Screen::Plan(..), Key::Up | Key::Char('k')) => Some(AppCommand::SelectPrevious),
        (Screen::Plan(..), Key::Down | Key::Char('j')) => Some(AppCommand::SelectNext),
        (Screen::Plan(..), Key::Char(',')) => Some(AppCommand::TogglePlanFocusedView),
        (Screen::Plan(..), Key::Tab) => Some(AppCommand::NavigatePlanMonth { forward: true }),
        (Screen::Plan(..), Key::BackTab) => Some(AppCommand::NavigatePlanMonth { forward: false }),
        (Screen::Plan(plan_state), Key::Char('e')) => {
            // Edit budgeted amount - only in Normal mode with valid selection
            if plan_state.input_mode == InputMode::Normal {
                let selected_idx = plan_state.table_state.borrow().selected()?;
                let visible_categories = plan_state.filtered_categories();

                if selected_idx < visible_categories.len() {
                    let category = visible_categories[selected_idx];
                    Some(AppCommand::InitiateBudgetEdit {
                        category_id: category.id.to_string(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        (Screen::Plan(..), Key::Char('r')) => {
            // Force refresh plan
            state
                .current_budget_id
                .as_ref()
                .map(|budget_id| AppCommand::LoadPlan {
                    budget_id: budget_id.clone(),
                    force_refresh: true,
                })
        }

        // Logs screen
        (Screen::Logs(..), Key::Up | Key::Char('k')) => Some(AppCommand::ScrollLogsUp),
        (Screen::Logs(..), Key::Down | Key::Char('j')) => Some(AppCommand::ScrollLogsDown),
        (Screen::Logs(..), Key::PageUp) => Some(AppCommand::ScrollLogsPageUp),
        (Screen::Logs(..), Key::PageDown) => Some(AppCommand::ScrollLogsPageDown),

        // Ignore other keys
        _ => None,
    }
}

/// Handle keyboard input when in transaction form mode
fn handle_transaction_form_keys(
    event: KeyEvent,
    trans_state: &TransactionsState,
) -> Option<AppCommand> {
    let key = event.key;

    // Ctrl+L to clear current field
    if event.modifiers.ctrl && matches!(key, Key::Char('l')) {
        return Some(AppCommand::ClearFormField);
    }

    // Ctrl+S to enter split mode (only when not already in split mode)
    if event.modifiers.ctrl && matches!(key, Key::Char('s')) {
        if let Some(ref form) = trans_state.form_state {
            if !form.is_split_mode {
                return Some(AppCommand::EnterSplitMode);
            }
        }
    }

    // Ctrl+N to add subtransaction (only in split mode)
    if event.modifiers.ctrl && matches!(key, Key::Char('n')) {
        if let Some(ref form) = trans_state.form_state {
            if form.is_split_mode {
                return Some(AppCommand::AddSubtransaction);
            }
        }
    }

    // Ctrl+D to delete subtransaction (only in split mode)
    if event.modifiers.ctrl && matches!(key, Key::Char('d')) {
        if let Some(ref form) = trans_state.form_state {
            if form.is_split_mode {
                return Some(AppCommand::DeleteSubtransaction);
            }
        }
    }

    match key {
        // Escape to cancel and close form
        Key::Esc => Some(AppCommand::ExitTransactionCreateMode),

        // Tab to navigate to next field
        Key::Tab => Some(AppCommand::NavigateFormField { forward: true }),

        // Shift+Tab to navigate to previous field
        Key::BackTab => Some(AppCommand::NavigateFormField { forward: false }),

        // Backspace to delete character
        Key::Backspace => Some(AppCommand::DeleteFormFieldChar),

        // Arrow keys for autocomplete navigation (when on Account, Payee, or Category field)
        Key::Up => {
            if let Some(ref form) = trans_state.form_state {
                // Check subtransaction category autocomplete first
                if let Some(sub_idx) = form.active_subtransaction_index {
                    if form.subtransaction_field == SubTransactionField::Category {
                        if let Some(sub) = form.subtransactions.get(sub_idx) {
                            if !sub.filtered_categories.is_empty() {
                                return Some(AppCommand::SelectAutocompleteItem { up: true });
                            }
                        }
                    }
                    return None;
                }

                match form.current_field {
                    Some(FormField::Payee) if !form.filtered_payees.is_empty() => {
                        Some(AppCommand::SelectAutocompleteItem { up: true })
                    }
                    Some(FormField::Category) if !form.filtered_categories.is_empty() => {
                        Some(AppCommand::SelectAutocompleteItem { up: true })
                    }
                    _ => None,
                }
            } else {
                None
            }
        }

        Key::Down => {
            if let Some(ref form) = trans_state.form_state {
                // Check subtransaction category autocomplete first
                if let Some(sub_idx) = form.active_subtransaction_index {
                    if form.subtransaction_field == SubTransactionField::Category {
                        if let Some(sub) = form.subtransactions.get(sub_idx) {
                            if !sub.filtered_categories.is_empty() {
                                return Some(AppCommand::SelectAutocompleteItem { up: false });
                            }
                        }
                    }
                    return None;
                }

                match form.current_field {
                    Some(FormField::Payee) if !form.filtered_payees.is_empty() => {
                        Some(AppCommand::SelectAutocompleteItem { up: false })
                    }
                    Some(FormField::Category) if !form.filtered_categories.is_empty() => {
                        Some(AppCommand::SelectAutocompleteItem { up: false })
                    }
                    _ => None,
                }
            } else {
                None
            }
        }

        // Enter key behavior depends on context
        Key::Enter => {
            if let Some(ref form) = trans_state.form_state {
                // Check subtransaction category autocomplete
                if let Some(sub_idx) = form.active_subtransaction_index {
                    if form.subtransaction_field == SubTransactionField::Category {
                        if let Some(sub) = form.subtransactions.get(sub_idx) {
                            if !sub.filtered_categories.is_empty() {
                                return Some(AppCommand::ConfirmAutocompleteSelection);
                            }
                        }
                    }
                }

                // Check for "Split" entry in category field
                if form.current_field == Some(FormField::Category)
                    && form.category.eq_ignore_ascii_case("split")
                {
                    return Some(AppCommand::ConfirmAutocompleteSelection);
                }

                if form.is_autocomplete_value_focused() {
                    Some(AppCommand::ConfirmAutocompleteSelection)
                } else {
                    Some(AppCommand::SubmitTransactionForm)
                }
            } else {
                None
            }
        }

        // Regular character input
        Key::Char(c) => Some(AppCommand::AppendFormFieldChar { c }),

        // Ignore other keys
        _ => None,
    }
}

/// Handle keyboard input when in delete confirmation mode
fn handle_delete_confirmation_keys(
    key: Key,
    trans_state: &TransactionsState,
    state: &AppState,
) -> Option<AppCommand> {
    match key {
        // Confirm deletion with 'y'
        Key::Char('y') | Key::Char('Y') => {
            if let Some(ref transaction_id) = trans_state.delete_confirmation_transaction_id {
                state.current_budget_id.as_ref().map(|budget_id| {
                    AppCommand::ConfirmTransactionDelete {
                        transaction_id: transaction_id.clone(),
                        budget_id: budget_id.clone(),
                    }
                })
            } else {
                None
            }
        }

        // Any other key cancels
        _ => Some(AppCommand::CancelTransactionDelete),
    }
}

/// Handle keyboard input when in reconciled edit confirmation mode
fn handle_reconciled_edit_confirmation_keys(
    key: Key,
    trans_state: &TransactionsState,
) -> Option<AppCommand> {
    match key {
        // Confirm edit with 'y'
        Key::Char('y') | Key::Char('Y') => {
            trans_state
                .reconciled_edit_transaction_id
                .as_ref()
                .map(|transaction_id| AppCommand::ConfirmReconciledEdit {
                    transaction_id: transaction_id.clone(),
                })
        }

        // Any other key cancels
        _ => Some(AppCommand::CancelReconciledEdit),
    }
}

/// Handle keyboard input when in reconcile confirmation mode
fn handle_reconcile_confirmation_keys(key: Key, state: &AppState) -> Option<AppCommand> {
    match key {
        // Confirm reconciliation with 'y'
        Key::Char('y') | Key::Char('Y') => {
            if let (Some(budget_id), Some(account_id)) =
                (&state.current_budget_id, &state.current_account_id)
            {
                Some(AppCommand::ConfirmReconcile {
                    budget_id: budget_id.clone(),
                    account_id: account_id.clone(),
                })
            } else {
                None
            }
        }

        // Any other key cancels
        _ => Some(AppCommand::CancelReconcile),
    }
}

/// Handle keyboard input when in budget edit mode on plan screen
fn handle_budget_edit_keys(event: KeyEvent, state: &AppState) -> Option<AppCommand> {
    let key = event.key;

    // Ctrl+L to clear field
    if event.modifiers.ctrl && matches!(key, Key::Char('l')) {
        return Some(AppCommand::ClearFormField);
    }

    match key {
        // Escape to cancel and exit edit mode
        Key::Esc => Some(AppCommand::ExitBudgetEditMode),

        // Enter to submit
        Key::Enter => {
            if let (Some(budget_id), Screen::Plan(plan_state)) =
                (&state.current_budget_id, state.current_screen())
            {
                plan_state
                    .month
                    .as_ref()
                    .map(|month_detail| AppCommand::SubmitBudgetEdit {
                        budget_id: budget_id.clone(),
                        month: month_detail.month.clone(),
                    })
            } else {
                None
            }
        }

        // Backspace to delete character
        Key::Backspace => Some(AppCommand::DeleteBudgetChar),

        // Character input: digits, decimal, negative, and math operators
        Key::Char(c)
            if c.is_ascii_digit()
                || c == '.'
                || c == '-'
                || c == '+'
                || c == '*'
                || c == '/'
                || c == '('
                || c == ')' =>
        {
            Some(AppCommand::AppendBudgetChar(c))
        }

        // Ignore other keys
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use ynab_api::endpoints::{
        accounts::{Account, AccountType},
        budgets::BudgetSummary,
    };

    /// Generate a deterministic UUID from a string ID for testing
    fn test_uuid(id: &str) -> uuid::Uuid {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        id.hash(&mut hasher);
        let hash = hasher.finish();
        let bytes = [
            (hash >> 56) as u8,
            (hash >> 48) as u8,
            (hash >> 40) as u8,
            (hash >> 32) as u8,
            (hash >> 24) as u8,
            (hash >> 16) as u8,
            (hash >> 8) as u8,
            hash as u8,
            (hash >> 56) as u8,
            (hash >> 48) as u8,
            (hash >> 40) as u8,
            (hash >> 32) as u8,
            (hash >> 24) as u8,
            (hash >> 16) as u8,
            (hash >> 8) as u8,
            hash as u8,
        ];
        uuid::Uuid::from_bytes(bytes)
    }

    /// Helper to create a default UI state on Budgets screen
    fn budgets_state() -> AppState {
        let mut state = AppState::new();
        state.history = vec![Screen::Budgets(BudgetsState {
            budgets: vec![BudgetSummary {
                id: test_uuid("budget1").into(),
                name: "Budget 1".to_string(),
                last_modified_on: None,
                first_month: None,
                last_month: None,
                date_format: None,
                currency_format: None,
                accounts: None,
            }],
            budgets_loading: LoadingState::Loaded,
            selected_budget_index: 0,
        })];
        state
    }

    /// Helper to create UI state on Accounts screen
    fn accounts_state() -> AppState {
        let mut state = AppState::new();
        state.current_budget_id = Some(test_uuid("budget1").to_string());
        state.history = vec![Screen::Accounts(AccountsState {
            accounts: vec![Account {
                id: test_uuid("account1"),
                name: "Checking".to_string(),
                account_type: AccountType::Checking,
                on_budget: true,
                closed: false,
                note: None,
                balance: 100000.into(),
                cleared_balance: 50000.into(),
                uncleared_balance: 50000.into(),
                transfer_payee_id: None,
                direct_import_linked: false,
                direct_import_in_error: false,
                deleted: false,
            }],
            accounts_loading: LoadingState::Loaded,
            table_state: RefCell::new(ratatui::widgets::TableState::default()),
            input_mode: InputMode::Normal,
            filter_query: String::new(),
            show_closed_accounts: false,
        })];
        state
    }

    // ============================================================================
    // Global Commands
    // ============================================================================

    #[test]
    fn test_quit_command() {
        let state = budgets_state();
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('q')), &state),
            Some(AppCommand::Quit)
        );
    }

    #[test]
    fn test_help_toggle() {
        let state = budgets_state();
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('?')), &state),
            Some(AppCommand::ToggleHelp)
        );
    }

    #[test]
    fn test_help_visible_blocks_other_commands() {
        let mut state = budgets_state();
        state.help_visible = true;

        // When help is visible, most keys should be ignored
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('j')), &state),
            None
        );
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('k')), &state),
            None
        );

        // Only '?', 'Esc', and 'q' should work
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('?')), &state),
            Some(AppCommand::ToggleHelp)
        );
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Esc), &state),
            Some(AppCommand::ToggleHelp)
        );
    }

    // ============================================================================
    // Multi-key Sequences
    // ============================================================================

    #[test]
    fn test_g_sets_pending_key() {
        let state = budgets_state();
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('g')), &state),
            Some(AppCommand::SetPendingKey('g'))
        );
    }

    #[test]
    fn test_gg_navigates_to_top() {
        let mut state = budgets_state();
        state.pending_key = Some('g');

        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('g')), &state),
            Some(AppCommand::NavigateToTop)
        );
    }

    #[test]
    fn test_gb_loads_budgets() {
        let mut state = budgets_state();
        state.pending_key = Some('g');

        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('b')), &state),
            Some(AppCommand::LoadBudgets {
                force_refresh: false,
                load_accounts: false
            })
        );
    }

    #[test]
    fn test_invalid_multi_key_sequence_clears_pending() {
        let mut state = budgets_state();
        state.pending_key = Some('g');

        // Any key that's not part of a valid sequence should clear pending
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('x')), &state),
            Some(AppCommand::ClearPendingKey)
        );
    }

    #[test]
    fn test_capital_g_navigates_to_bottom() {
        let state = budgets_state();
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('G')), &state),
            Some(AppCommand::NavigateToBottom)
        );
    }

    // ============================================================================
    // Navigation Commands
    // ============================================================================

    #[test]
    fn test_navigate_back() {
        let state = budgets_state();
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Left), &state),
            Some(AppCommand::NavigateBack)
        );
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('h')), &state),
            Some(AppCommand::NavigateBack)
        );
    }

    // ============================================================================
    // Budgets Screen Commands
    // ============================================================================

    #[test]
    fn test_budgets_screen_select_next() {
        let state = budgets_state();
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Down), &state),
            Some(AppCommand::SelectNext)
        );
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('j')), &state),
            Some(AppCommand::SelectNext)
        );
    }

    #[test]
    fn test_budgets_screen_enter_loads_accounts() {
        let state = budgets_state();
        let result = handle_key_input(KeyEvent::new(Key::Enter), &state);

        // Extract the command to check its contents
        if let Some(AppCommand::LoadAccounts {
            budget_id,
            budget,
            force_refresh,
        }) = result
        {
            assert_eq!(budget_id, test_uuid("budget1").to_string());
            assert!(budget.is_some());
            assert_eq!(
                (*budget).as_ref().unwrap().id,
                ynab_api::endpoints::BudgetId::from(test_uuid("budget1"))
            );
            assert!(!force_refresh);
        } else {
            panic!("Expected LoadAccounts command");
        }
    }

    // ============================================================================
    // Accounts Screen Commands
    // ============================================================================

    #[test]
    fn test_accounts_screen_slash_enters_filter_mode() {
        let state = accounts_state();
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('/')), &state),
            Some(AppCommand::EnterFilterMode)
        );
    }

    #[test]
    fn test_accounts_screen_esc_with_help_visible_closes_help() {
        let mut state = accounts_state();
        state.help_visible = true;

        // Help has higher priority than clear filter
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Esc), &state),
            Some(AppCommand::ToggleHelp)
        );
    }

    // ============================================================================
    // Filter Mode Commands
    // ============================================================================

    #[test]
    fn test_filter_mode_char_appends() {
        let mut state = accounts_state();
        if let Screen::Accounts(ref mut accounts) = state.history[0] {
            accounts.input_mode = InputMode::Filter;
        }

        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Char('t')), &state),
            Some(AppCommand::AppendFilterChar('t'))
        );
    }

    #[test]
    fn test_filter_mode_enter_exits() {
        let mut state = accounts_state();
        if let Screen::Accounts(ref mut accounts) = state.history[0] {
            accounts.input_mode = InputMode::Filter;
        }

        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Enter), &state),
            Some(AppCommand::ExitFilterMode)
        );
    }

    // ============================================================================
    // Priority Testing
    // ============================================================================

    #[test]
    fn test_priority_help_overrides_screen_esc() {
        let mut state = accounts_state();
        state.help_visible = true;

        // Help should take priority over screen-specific Esc (clear filter)
        assert_eq!(
            handle_key_input(KeyEvent::new(Key::Esc), &state),
            Some(AppCommand::ToggleHelp)
        );
    }
}
