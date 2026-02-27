use crate::background::{data_loader::DataLoader, BackgroundTaskManager};
use crate::events::{AppCommand, DataEvent};
use crate::state::*;
use crate::ui::screens::Screen;
use crate::utils;
use ratatui::widgets::TableState;
use std::cell::RefCell;
use throbber_widgets_tui::ThrobberState;
use ynab_api::endpoints::transactions::{BulkTransactionUpdate, FlagColor, ReconciliationStatus};
use ynab_api::endpoints::{BudgetId, TransactionId};
use ynab_api::Request;

/// Execute a command by spawning background tasks or sending app events
pub fn execute_command(
    command: AppCommand,
    state: &mut AppState,
    task_manager: &mut BackgroundTaskManager,
    data_loader: &DataLoader,
) {
    // Save whether we're setting a pending key (we don't want to clear it in that case)
    let is_setting_pending_key = matches!(command, AppCommand::SetPendingKey(_));

    match command {
        AppCommand::SelectNext => {
            // Update selection based on current screen
            match state.current_screen_mut() {
                Screen::Budgets(budgets_state) => {
                    if !budgets_state.budgets.is_empty() {
                        budgets_state.selected_budget_index =
                            (budgets_state.selected_budget_index + 1) % budgets_state.budgets.len();
                    }
                }
                Screen::Accounts(accounts_state) => {
                    accounts_state.select_next();
                }
                Screen::Transactions(transactions_state) => {
                    transactions_state.select_next();
                }
                Screen::Plan(plan_state) => {
                    plan_state.select_next();
                }
                Screen::Logs(_) => {
                    // Logs screen uses its own scroll commands, not SelectNext
                }
            }
        }

        AppCommand::SelectPrevious => {
            // Update selection based on current screen
            match state.current_screen_mut() {
                Screen::Budgets(budgets_state) => {
                    if !budgets_state.budgets.is_empty() {
                        if budgets_state.selected_budget_index == 0 {
                            budgets_state.selected_budget_index = budgets_state.budgets.len() - 1;
                        } else {
                            budgets_state.selected_budget_index -= 1;
                        }
                    }
                }
                Screen::Accounts(accounts_state) => {
                    accounts_state.select_prev();
                }
                Screen::Transactions(transactions_state) => {
                    transactions_state.select_prev();
                }
                Screen::Plan(plan_state) => {
                    plan_state.select_prev();
                }
                Screen::Logs(_) => {
                    // Logs screen uses its own scroll commands, not SelectPrevious
                }
            }
        }

        AppCommand::LoadBudgets {
            force_refresh,
            load_accounts,
        } => {
            // Check if we're already on Budgets screen (refresh) or navigating to it (new)
            match state.current_screen_mut() {
                Screen::Budgets(budgets_state) => {
                    // Already on Budgets screen - just update loading state (refresh)
                    tracing::debug!("Refreshing budgets screen");
                    budgets_state.budgets_loading = LoadingState::Loading(ThrobberState::default());
                }
                _ => {
                    // Navigate to budgets screen
                    tracing::debug!("Navigating to budgets screen");
                    state.navigate_to(Screen::Budgets(BudgetsState {
                        budgets_loading: LoadingState::Loading(ThrobberState::default()),
                        ..Default::default()
                    }));
                }
            }

            // Spawn background task to load budgets
            let data_loader = data_loader.clone();
            let future = async move {
                data_loader.load_budgets(force_refresh, load_accounts).await;
            };

            task_manager.spawn_load_task("load_budgets".to_string(), future);
        }

        AppCommand::LoadAccounts {
            budget_id,
            budget,
            force_refresh,
        } => {
            // Update current budget ID and budget details
            state.current_budget_id = Some(budget_id.clone());
            if let Some(budget) = *budget {
                state.current_budget = Some(budget);
            }

            // Check if we're already on Accounts screen (refresh) or navigating to it (new)
            match state.current_screen_mut() {
                Screen::Accounts(accounts_state) => {
                    // Already on Accounts screen - just update loading state (refresh)
                    tracing::debug!("Refreshing accounts screen");
                    accounts_state.accounts_loading =
                        LoadingState::Loading(ThrobberState::default());
                }
                _ => {
                    // Navigate to accounts screen
                    tracing::debug!("Navigating to accounts screen");
                    state.navigate_to(Screen::Accounts(AccountsState {
                        accounts_loading: LoadingState::Loading(ThrobberState::default()),
                        ..Default::default()
                    }));
                }
            }

            // Spawn background task to load accounts
            let data_loader = data_loader.clone();
            let budget_id_clone = budget_id.clone();
            let future = async move {
                data_loader
                    .load_accounts(budget_id_clone, force_refresh)
                    .await;
            };

            task_manager.spawn_load_task(format!("load_accounts_{}", budget_id), future);
        }

        AppCommand::LoadTransactions {
            budget_id,
            account_id,
            force_refresh,
        } => {
            // Update current account ID
            state.current_account_id = Some(account_id.clone());

            // Check if we're already on Transactions screen (refresh) or navigating to it (new)
            match state.current_screen_mut() {
                Screen::Transactions(transactions_state) => {
                    // Already on Transactions screen - just update loading state (refresh)
                    tracing::debug!("Refreshing transactions screen");
                    transactions_state.transactions_loading =
                        LoadingState::Loading(ThrobberState::default());
                }
                _ => {
                    // Navigate to transactions screen
                    tracing::debug!("Navigating to transactions screen");
                    state.navigate_to(Screen::Transactions(Box::new(TransactionsState {
                        transactions_loading: LoadingState::Loading(ThrobberState::default()),
                        ..Default::default()
                    })));
                }
            }

            // Spawn background task to load transactions
            let data_loader = data_loader.clone();
            let budget_id_clone = budget_id.clone();
            let account_id_clone = account_id.clone();
            let future = async move {
                data_loader
                    .load_transactions(budget_id_clone, account_id_clone, force_refresh)
                    .await;
            };

            task_manager.spawn_load_task(
                format!("load_transactions_{}_{}", budget_id, account_id),
                future,
            );
        }

        AppCommand::LoadPlan {
            budget_id,
            force_refresh,
        } => {
            // Check if we're already on Plan screen (refresh) or navigating to it (new)
            match state.current_screen_mut() {
                Screen::Plan(plan_state) => {
                    // Already on Plan screen - just update loading state (refresh)
                    tracing::debug!("Refreshing plan screen");
                    plan_state.plan_loading = LoadingState::Loading(ThrobberState::default());
                }
                _ => {
                    // Navigate to plan screen
                    tracing::debug!("Navigating to plan screen");
                    state.navigate_to(Screen::Plan(PlanState {
                        plan_loading: LoadingState::Loading(ThrobberState::default()),
                        ..Default::default()
                    }));
                }
            }

            // Spawn background task to load plan
            let data_loader = data_loader.clone();
            let budget_id_clone = budget_id.clone();
            let future = async move {
                data_loader.load_plan(budget_id_clone, force_refresh).await;
            };

            task_manager.spawn_load_task(format!("load_plan_{}", budget_id), future);
        }

        AppCommand::LoadPlanMonth { budget_id, month } => {
            // Set loading state on existing Plan screen
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                plan_state.plan_loading = LoadingState::Loading(ThrobberState::default());
            }

            // Spawn background task to load specific month
            let data_loader = data_loader.clone();
            let budget_id_clone = budget_id.clone();
            let month_clone = month.clone();
            let future = async move {
                data_loader
                    .load_plan_month(budget_id_clone, month_clone)
                    .await;
            };

            task_manager.spawn_load_task(format!("load_plan_{}_{}", budget_id, month), future);
        }

        AppCommand::NavigatePlanMonth { forward } => {
            // Get the current month and compute next/previous
            if let Screen::Plan(plan_state) = state.current_screen() {
                if let Some(ref month_detail) = plan_state.month {
                    // Parse current month (format: YYYY-MM-DD)
                    if let Some(new_month) = compute_adjacent_month(&month_detail.month, forward) {
                        if let Some(budget_id) = &state.current_budget_id {
                            // Recursively execute LoadPlanMonth command
                            execute_command(
                                AppCommand::LoadPlanMonth {
                                    budget_id: budget_id.clone(),
                                    month: new_month,
                                },
                                state,
                                task_manager,
                                data_loader,
                            );
                        }
                    }
                }
            }
        }

        AppCommand::ToggleTransactionCleared {
            transaction_id,
            budget_id,
        } => {
            // Optimistic update: toggle cleared status locally
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                if let Some(transaction) = transactions_state
                    .transactions
                    .iter_mut()
                    .find(|t| t.id.to_string() == transaction_id)
                {
                    // Save original status for rollback if needed
                    let original_status = transaction.cleared;
                    let original_approved = transaction.approved;

                    // Toggle: cleared <-> uncleared (never touch reconciled)
                    let new_status = match transaction.cleared {
                        ReconciliationStatus::Cleared => ReconciliationStatus::Uncleared,
                        ReconciliationStatus::Uncleared => ReconciliationStatus::Cleared,
                        ReconciliationStatus::Reconciled => {
                            unreachable!("Should never be able to toggle reconciled transactions")
                        }
                    };

                    // Apply optimistic update
                    transaction.cleared = new_status;
                    transaction.approved = true;

                    tracing::info!(
                        "Optimistically toggled transaction {} from {} to {}",
                        transaction_id,
                        original_status,
                        new_status.clone()
                    );

                    // Spawn background task to update via API
                    let api_client = data_loader.api_client.clone();
                    let data_tx = data_loader.data_tx.clone();
                    let transaction_id_clone = transaction_id.clone();
                    let budget_id_clone = budget_id.clone();
                    let new_status_clone = new_status;

                    let future = async move {
                        let budget_id_api: BudgetId = budget_id_clone.clone().into();
                        let transaction_id: TransactionId = transaction_id_clone
                            .parse()
                            .expect("invalid transaction id");
                        let req = Request::transactions()
                            .with_budget(budget_id_api)
                            .update(transaction_id)
                            .cleared(new_status_clone)
                            .approved(true);

                        match api_client.send(req).await {
                            Ok(_) => {
                                tracing::info!(
                                    "Transaction {} updated successfully on server",
                                    transaction_id_clone
                                );
                                let _ = data_tx.send(DataEvent::TransactionUpdated {
                                    transaction_id: transaction_id_clone,
                                });
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to update transaction {}: {}",
                                    transaction_id_clone,
                                    e
                                );
                                let _ = data_tx.send(DataEvent::TransactionUpdateFailed {
                                    transaction_id: transaction_id_clone,
                                    original_status,
                                    original_approved,
                                    error: e.to_string(),
                                });
                            }
                        }
                    };

                    task_manager
                        .spawn_load_task(format!("update_transaction_{}", transaction_id), future);
                }
            }
        }

        AppCommand::EnterFilterMode => match state.current_screen_mut() {
            Screen::Transactions(trans_state) => {
                trans_state.input_mode = InputMode::Filter;
            }
            Screen::Accounts(accounts_state) => {
                accounts_state.input_mode = InputMode::Filter;
            }
            _ => {}
        },

        AppCommand::ExitFilterMode => {
            match state.current_screen_mut() {
                Screen::Transactions(trans_state) => {
                    trans_state.input_mode = InputMode::Normal;
                    // Keep filter_query intact - filter remains active
                }
                Screen::Accounts(accounts_state) => {
                    accounts_state.input_mode = InputMode::Normal;
                    // Keep filter_query intact - filter remains active
                }
                _ => {}
            }
        }

        AppCommand::AppendFilterChar(c) => {
            match state.current_screen_mut() {
                Screen::Transactions(trans_state) => {
                    trans_state.filter_query.push(c);
                    // Reset table selection when filter changes
                    trans_state.table_state = RefCell::new(TableState::default().with_selected(0));
                }
                Screen::Accounts(accounts_state) => {
                    accounts_state.filter_query.push(c);
                    // Reset table selection when filter changes
                    accounts_state.table_state =
                        RefCell::new(TableState::default().with_selected(0));
                }
                _ => {}
            }
        }

        AppCommand::DeleteFilterChar => {
            match state.current_screen_mut() {
                Screen::Transactions(trans_state) => {
                    trans_state.filter_query.pop();
                    // Reset table selection when filter changes
                    trans_state.table_state = RefCell::new(TableState::default().with_selected(0));
                }
                Screen::Accounts(accounts_state) => {
                    accounts_state.filter_query.pop();
                    // Reset table selection when filter changes
                    accounts_state.table_state =
                        RefCell::new(TableState::default().with_selected(0));
                }
                _ => {}
            }
        }

        AppCommand::ClearFilter => {
            match state.current_screen_mut() {
                Screen::Transactions(trans_state) => {
                    trans_state.filter_query.clear();
                    trans_state.input_mode = InputMode::Normal;
                    // Reset table selection
                    trans_state.table_state = RefCell::new(TableState::default().with_selected(0));
                }
                Screen::Accounts(accounts_state) => {
                    accounts_state.filter_query.clear();
                    accounts_state.input_mode = InputMode::Normal;
                    // Reset table selection
                    accounts_state.table_state =
                        RefCell::new(TableState::default().with_selected(0));
                }
                _ => {}
            }
        }

        AppCommand::ToggleShowClosedAccounts => {
            if let Screen::Accounts(accounts_state) = state.current_screen_mut() {
                accounts_state.show_closed_accounts = !accounts_state.show_closed_accounts;
                // Reset table selection when toggling view
                accounts_state.table_state = RefCell::new(TableState::default().with_selected(0));
            }
        }

        AppCommand::ToggleShowReconciledTransactions => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions_state.show_reconciled_transactions =
                    !transactions_state.show_reconciled_transactions;
                // Reset table selection when toggling view
                transactions_state.table_state =
                    RefCell::new(TableState::default().with_selected(0));
            }
        }

        AppCommand::TogglePlanFocusedView => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                plan_state.focused_view = plan_state.focused_view.next();
                // Reset table selection when toggling view
                plan_state.table_state = RefCell::new(TableState::default().with_selected(0));
            }
        }

        AppCommand::ToggleHelp => {
            state.help_visible = !state.help_visible;
        }

        AppCommand::NavigateToTop => {
            // Navigate to the first item in the current screen's list
            match state.current_screen_mut() {
                Screen::Budgets(budgets_state) => {
                    if !budgets_state.budgets.is_empty() {
                        budgets_state.selected_budget_index = 0;
                    }
                }
                Screen::Accounts(accounts_state) => {
                    let num_items = accounts_state.filtered_accounts().len();
                    if num_items > 0 {
                        accounts_state.table_state =
                            RefCell::new(TableState::default().with_selected(0));
                    }
                }
                Screen::Transactions(transactions_state) => {
                    let num_items = transactions_state.filtered_transactions().len();
                    if num_items > 0 {
                        transactions_state.table_state =
                            RefCell::new(TableState::default().with_selected(0));
                    }
                }
                Screen::Plan(plan_state) => {
                    let num_items = plan_state.filtered_categories().len();
                    if num_items > 0 {
                        plan_state.table_state =
                            RefCell::new(TableState::default().with_selected(0));
                    }
                }
                Screen::Logs(logs_state) => {
                    // Scroll to oldest logs (top)
                    logs_state.scroll_offset = logs_state.total_entries.saturating_sub(1);
                }
            }
        }

        AppCommand::NavigateToBottom => {
            // Navigate to the last item in the current screen's list
            match state.current_screen_mut() {
                Screen::Budgets(budgets_state) => {
                    if !budgets_state.budgets.is_empty() {
                        budgets_state.selected_budget_index = budgets_state.budgets.len() - 1;
                    }
                }
                Screen::Accounts(accounts_state) => {
                    let num_items = accounts_state.filtered_accounts().len();
                    if num_items > 0 {
                        accounts_state.table_state =
                            RefCell::new(TableState::default().with_selected(num_items - 1));
                    }
                }
                Screen::Transactions(transactions_state) => {
                    let num_items = transactions_state.filtered_transactions().len();
                    if num_items > 0 {
                        transactions_state.table_state =
                            RefCell::new(TableState::default().with_selected(num_items - 1));
                    }
                }
                Screen::Plan(plan_state) => {
                    let num_items = plan_state.filtered_categories().len();
                    if num_items > 0 {
                        plan_state.table_state =
                            RefCell::new(TableState::default().with_selected(num_items - 1));
                    }
                }
                Screen::Logs(logs_state) => {
                    // Scroll to newest logs (bottom)
                    logs_state.scroll_offset = 0;
                }
            }
        }

        AppCommand::SetPendingKey(c) => {
            state.pending_key = Some(c);
        }

        AppCommand::ClearPendingKey => {
            state.pending_key = None;
        }

        // Transaction creation form commands
        AppCommand::EnterTransactionCreateMode => {
            // Get IDs and date format before mutable borrow
            let account_id_opt = state.current_account_id.clone();
            let budget_id_opt = state.current_budget_id.clone();
            let date_format = state
                .current_budget
                .as_ref()
                .and_then(|b| b.date_format.as_ref())
                .map(|d| d.format.clone())
                .unwrap_or_else(|| "YYYY-MM-DD".to_string());

            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                // Initialize form state if we have an account ID
                if let Some(account_id) = account_id_opt {
                    trans_state.table_state.borrow_mut().select_first();
                    trans_state.input_mode = InputMode::TransactionForm;
                    trans_state.form_state =
                        Some(TransactionFormState::new(account_id, &date_format));

                    // Load payees and categories if not already loaded
                    if let Some(budget_id) = budget_id_opt {
                        if trans_state.payees.is_empty() {
                            let data_loader = data_loader.clone();
                            let budget_id_clone = budget_id.clone();
                            let future = async move {
                                data_loader.load_payees(budget_id_clone, false).await;
                            };
                            task_manager.spawn_load_task("load_payees".to_string(), future);
                        }

                        if trans_state.categories.is_empty() {
                            let data_loader = data_loader.clone();
                            let future = async move {
                                data_loader.load_categories(budget_id, false).await;
                            };
                            task_manager.spawn_load_task("load_categories".to_string(), future);
                        }
                    }
                }
            }
        }

        AppCommand::ExitTransactionCreateMode => {
            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                trans_state.input_mode = InputMode::Normal;
                trans_state.form_state = None;
            }
        }

        AppCommand::NavigateFormField { forward } => {
            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                if let Some(ref mut form) = trans_state.form_state {
                    use FormField::*;

                    // If leaving the Amount field, evaluate any math expression
                    if form.current_field == Some(Amount) && !form.amount.is_empty() {
                        if let Some(result) = utils::math::evaluate_expression(&form.amount) {
                            form.amount = result;
                        }
                    }

                    // Handle split mode navigation
                    if form.is_split_mode {
                        if let Some(sub_idx) = form.active_subtransaction_index {
                            // Currently in a subtransaction
                            // If leaving the subtransaction Amount field, evaluate any math expression
                            if form.subtransaction_field == SubTransactionField::Amount {
                                let sub_amount = &form.subtransactions[sub_idx].amount;
                                if !sub_amount.is_empty() {
                                    if let Some(result) =
                                        utils::math::evaluate_expression(sub_amount)
                                    {
                                        form.subtransactions[sub_idx].amount = result;
                                    }
                                }
                            }

                            if forward {
                                match form.subtransaction_field {
                                    SubTransactionField::Category => {
                                        form.subtransaction_field = SubTransactionField::Memo;
                                    }
                                    SubTransactionField::Memo => {
                                        form.subtransaction_field = SubTransactionField::Amount;
                                    }
                                    SubTransactionField::Amount => {
                                        // Move to next subtransaction or exit to main memo
                                        if sub_idx + 1 < form.subtransactions.len() {
                                            form.active_subtransaction_index = Some(sub_idx + 1);
                                            form.subtransaction_field =
                                                SubTransactionField::Category;
                                        } else {
                                            // Exit subtransaction editing, go to main transaction
                                            form.active_subtransaction_index = None;
                                            form.current_field = Some(FlagColor);
                                        }
                                    }
                                }
                            } else {
                                // Navigate backward
                                match form.subtransaction_field {
                                    SubTransactionField::Amount => {
                                        form.subtransaction_field = SubTransactionField::Memo;
                                    }
                                    SubTransactionField::Memo => {
                                        form.subtransaction_field = SubTransactionField::Category;
                                    }
                                    SubTransactionField::Category => {
                                        if sub_idx > 0 {
                                            form.active_subtransaction_index = Some(sub_idx - 1);
                                            form.subtransaction_field = SubTransactionField::Amount;
                                        } else {
                                            form.active_subtransaction_index = None;
                                            form.current_field = Some(Cleared);
                                        }
                                    }
                                }
                            }
                            form.validation_error = None;
                            return;
                        } else {
                            // Not in a subtransaction, but in split mode
                            if forward && form.current_field == Some(Cleared) {
                                // Enter first subtransaction
                                form.current_field = None;
                                form.active_subtransaction_index = Some(0);
                                form.subtransaction_field = SubTransactionField::Category;
                                form.validation_error = None;
                                return;
                            } else if !forward && form.current_field == Some(FlagColor) {
                                // Go back to last subtransaction
                                if !form.subtransactions.is_empty() {
                                    form.current_field = None;
                                    form.active_subtransaction_index =
                                        Some(form.subtransactions.len() - 1);
                                    form.subtransaction_field = SubTransactionField::Amount;
                                    form.validation_error = None;
                                    return;
                                }
                            }
                        }
                    }

                    // Normal form navigation (non-split mode or main fields)
                    form.current_field = if forward {
                        match form.current_field {
                            Some(FlagColor) => Some(Date),
                            Some(Date) => Some(Payee),
                            Some(Payee) => Some(Category),
                            Some(Category) => Some(Memo),
                            Some(Memo) => Some(Amount),
                            Some(Amount) => Some(Cleared),
                            Some(Cleared) => Some(FlagColor), // Wrap around
                            None => Some(FlagColor),
                        }
                    } else {
                        match form.current_field {
                            Some(FlagColor) => Some(Cleared), // Wrap around
                            Some(Cleared) => Some(Amount),
                            Some(Amount) => Some(Memo),
                            Some(Memo) => Some(Category),
                            Some(Category) => Some(Payee),
                            Some(Payee) => Some(Date),
                            Some(Date) => Some(FlagColor),
                            None => Some(Cleared),
                        }
                    };
                    // Clear validation error when navigating
                    form.validation_error = None;
                }
            }
        }

        AppCommand::AppendFormFieldChar { c } => {
            // Get date format before mutable borrow
            let date_format = state
                .current_budget
                .as_ref()
                .and_then(|b| b.date_format.as_ref())
                .map(|d| d.format.clone())
                .unwrap_or_else(|| "YYYY-MM-DD".to_string());

            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                if let Some(ref mut form) = trans_state.form_state {
                    // Handle subtransaction input if active
                    if let Some(sub_idx) = form.active_subtransaction_index {
                        if let Some(sub) = form.subtransactions.get_mut(sub_idx) {
                            match form.subtransaction_field {
                                SubTransactionField::Amount => {
                                    // Allow digits, decimal point, and math operators
                                    if c.is_ascii_digit()
                                        || c == '.'
                                        || c == '-'
                                        || c == '+'
                                        || c == '*'
                                        || c == '/'
                                        || c == '('
                                        || c == ')'
                                    {
                                        sub.amount.push(c);
                                    }
                                }
                                SubTransactionField::Category => {
                                    sub.category.push(c);
                                    // Update autocomplete for subtransaction
                                    sub.filtered_categories = autocomplete::filter_categories(
                                        &trans_state.categories,
                                        &sub.category,
                                    );
                                    sub.category_selection_index = 0;
                                }
                                SubTransactionField::Memo => {
                                    sub.memo.push(c);
                                }
                            }
                        }
                        form.validation_error = None;
                        return;
                    }

                    // Append character to current field
                    match form.current_field {
                        Some(FormField::Date) => {
                            if let Some(new_date) =
                                utils::dates::append_date_char(&form.date, c, &date_format)
                            {
                                form.date = new_date;
                            }
                        }
                        Some(FormField::Amount) => {
                            // Allow digits, decimal point, and math operators
                            if c.is_ascii_digit()
                                || c == '.'
                                || c == '-'
                                || c == '+'
                                || c == '*'
                                || c == '/'
                                || c == '('
                                || c == ')'
                            {
                                form.amount.push(c);
                            }
                        }
                        Some(FormField::Payee) => {
                            form.payee.push(c);
                            // Update autocomplete
                            form.filtered_payees =
                                autocomplete::filter_payees(&trans_state.payees, &form.payee);
                            form.payee_selection_index = 0;
                        }
                        Some(FormField::Category) => {
                            // If in split mode, typing exits split mode
                            if form.is_split_mode {
                                form.is_split_mode = false;
                                form.subtransactions.clear();
                                form.active_subtransaction_index = None;
                            }
                            form.category.push(c);
                            // Update autocomplete
                            form.filtered_categories = autocomplete::filter_categories(
                                &trans_state.categories,
                                &form.category,
                            );
                            form.category_selection_index = 0;
                        }
                        Some(FormField::Memo) => form.memo.push(c),
                        Some(FormField::FlagColor) => {
                            use FlagColor::*;
                            form.flag_color = match form.flag_color {
                                None => Some(Red),
                                Some(Red) => Some(Orange),
                                Some(Orange) => Some(Yellow),
                                Some(Yellow) => Some(Green),
                                Some(Green) => Some(Blue),
                                Some(Blue) => Some(Purple),
                                Some(Purple) => None,
                            }
                        }
                        Some(FormField::Cleared) => {
                            // Cycle through cleared options: uncleared -> cleared -> reconciled
                            match form.cleared {
                                ReconciliationStatus::Uncleared => {
                                    form.cleared = ReconciliationStatus::Cleared
                                }
                                ReconciliationStatus::Cleared => {
                                    form.cleared = ReconciliationStatus::Uncleared
                                }
                                ReconciliationStatus::Reconciled => {}
                            };
                        }
                        None => {}
                    }
                    // Clear validation error when typing
                    form.validation_error = None;
                }
            }
        }

        AppCommand::DeleteFormFieldChar => {
            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                if let Some(ref mut form) = trans_state.form_state {
                    // Handle subtransaction input if active
                    if let Some(sub_idx) = form.active_subtransaction_index {
                        if let Some(sub) = form.subtransactions.get_mut(sub_idx) {
                            match form.subtransaction_field {
                                SubTransactionField::Amount => {
                                    sub.amount.pop();
                                }
                                SubTransactionField::Category => {
                                    sub.category.pop();
                                    // Update autocomplete for subtransaction
                                    sub.filtered_categories = autocomplete::filter_categories(
                                        &trans_state.categories,
                                        &sub.category,
                                    );
                                    sub.category_selection_index = 0;
                                }
                                SubTransactionField::Memo => {
                                    sub.memo.pop();
                                }
                            }
                        }
                        return;
                    }

                    // Delete last character from current field
                    match form.current_field {
                        Some(FormField::Date) => {
                            form.date.pop();
                        }
                        Some(FormField::Amount) => {
                            form.amount.pop();
                        }
                        Some(FormField::Payee) => {
                            form.payee.pop();
                            // Update autocomplete
                            form.filtered_payees =
                                autocomplete::filter_payees(&trans_state.payees, &form.payee);
                            form.payee_selection_index = 0;
                        }
                        Some(FormField::Category) => {
                            form.category.pop();
                            // Update autocomplete
                            form.filtered_categories = autocomplete::filter_categories(
                                &trans_state.categories,
                                &form.category,
                            );
                            form.category_selection_index = 0;
                        }
                        Some(FormField::Memo) => {
                            form.memo.pop();
                        }
                        Some(FormField::FlagColor) | Some(FormField::Cleared) => {
                            // No-op for these fields (they cycle, not type)
                        }
                        None => {}
                    }
                }
            }
        }

        AppCommand::ClearFormField => {
            match state.current_screen_mut() {
                Screen::Transactions(trans_state) => {
                    if let Some(ref mut form) = trans_state.form_state {
                        // Clear the current field
                        match form.current_field {
                            Some(FormField::Date) => {
                                form.date.clear();
                            }
                            Some(FormField::Amount) => {
                                form.amount.clear();
                            }
                            Some(FormField::Payee) => {
                                form.payee.clear();
                                // Update autocomplete
                                form.filtered_payees =
                                    autocomplete::filter_payees(&trans_state.payees, &form.payee);
                                form.payee_selection_index = 0;
                            }
                            Some(FormField::Category) => {
                                form.category.clear();
                                // Update autocomplete
                                form.filtered_categories = autocomplete::filter_categories(
                                    &trans_state.categories,
                                    &form.category,
                                );
                                form.category_selection_index = 0;
                            }
                            Some(FormField::Memo) => {
                                form.memo.clear();
                            }
                            Some(FormField::FlagColor) => {
                                form.flag_color = None;
                            }
                            Some(FormField::Cleared) => {
                                form.cleared = ReconciliationStatus::Uncleared;
                            }
                            None => {}
                        }
                    }
                }
                Screen::Plan(plan_state) => {
                    if let Some(ref mut form) = plan_state.budget_form {
                        form.budgeted_input.clear();
                        form.validation_error = None;
                    }
                }
                _ => {}
            }
        }

        AppCommand::SelectAutocompleteItem { up } => {
            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                if let Some(ref mut form) = trans_state.form_state {
                    // Handle subtransaction category autocomplete
                    if let Some(sub_idx) = form.active_subtransaction_index {
                        if form.subtransaction_field == SubTransactionField::Category {
                            if let Some(sub) = form.subtransactions.get_mut(sub_idx) {
                                let len = sub.filtered_categories.len();
                                if len > 0 {
                                    if up {
                                        sub.category_selection_index =
                                            if sub.category_selection_index == 0 {
                                                len - 1
                                            } else {
                                                sub.category_selection_index - 1
                                            };
                                    } else {
                                        sub.category_selection_index =
                                            (sub.category_selection_index + 1) % len;
                                    }
                                }
                            }
                        }
                        return;
                    }

                    match form.current_field {
                        Some(FormField::Payee) => {
                            let len = form.filtered_payees.len();
                            if len > 0 {
                                if up {
                                    form.payee_selection_index = if form.payee_selection_index == 0
                                    {
                                        len - 1
                                    } else {
                                        form.payee_selection_index - 1
                                    };
                                } else {
                                    form.payee_selection_index =
                                        (form.payee_selection_index + 1) % len;
                                }
                            }
                        }
                        Some(FormField::Category) => {
                            let len = form.filtered_categories.len();
                            if len > 0 {
                                if up {
                                    form.category_selection_index =
                                        if form.category_selection_index == 0 {
                                            len - 1
                                        } else {
                                            form.category_selection_index - 1
                                        };
                                } else {
                                    form.category_selection_index =
                                        (form.category_selection_index + 1) % len;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        AppCommand::ConfirmAutocompleteSelection => {
            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                if let Some(ref mut form) = trans_state.form_state {
                    match form.current_field {
                        Some(FormField::Payee) => {
                            if let Some(payee) =
                                form.filtered_payees.get(form.payee_selection_index)
                            {
                                form.payee = payee.name.clone();
                                form.filtered_payees.clear();
                            }
                        }
                        Some(FormField::Category) => {
                            // Check if user is entering split mode
                            if form.category.eq_ignore_ascii_case("split") {
                                form.is_split_mode = true;
                                form.subtransactions.push(SubTransactionFormState::new());
                                form.category.clear();
                                form.filtered_categories.clear();
                            } else if let Some(category) =
                                form.filtered_categories.get(form.category_selection_index)
                            {
                                form.category = category.name.clone();
                                form.filtered_categories.clear();
                            }
                        }
                        _ => {}
                    }

                    // Handle subtransaction category autocomplete confirmation
                    if let Some(sub_idx) = form.active_subtransaction_index {
                        if form.subtransaction_field == SubTransactionField::Category {
                            if let Some(sub) = form.subtransactions.get_mut(sub_idx) {
                                if let Some(category) =
                                    sub.filtered_categories.get(sub.category_selection_index)
                                {
                                    sub.category = category.name.clone();
                                    sub.filtered_categories.clear();
                                }
                            }
                        }
                    }
                }
            }
        }

        AppCommand::SubmitTransactionForm => {
            // Get budget ID and date format before mutable borrow
            let budget_id_opt = state.current_budget_id.clone();
            let date_format = state
                .current_budget
                .as_ref()
                .and_then(|b| b.date_format.as_ref())
                .map(|d| d.format.clone())
                .unwrap_or_else(|| "YYYY-MM-DD".to_string());

            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                if let Some(ref form) = trans_state.form_state {
                    // Check if editing or creating
                    if let Some(ref transaction_id) = form.editing_transaction_id {
                        // EDIT MODE - Build update request
                        match validators::build_transaction_update(
                            form,
                            &trans_state.payees,
                            &trans_state.categories,
                            &date_format,
                        ) {
                            Ok(update_request) => {
                                if let Some(budget_id) = budget_id_opt {
                                    let data_loader = data_loader.clone();
                                    let transaction_id_clone = transaction_id.clone();
                                    let future = async move {
                                        data_loader
                                            .update_transaction_full(
                                                budget_id,
                                                transaction_id_clone,
                                                update_request,
                                            )
                                            .await;
                                    };
                                    task_manager
                                        .spawn_load_task("update_transaction".to_string(), future);
                                }
                            }
                            Err(error) => {
                                // Set validation error
                                if let Some(ref mut form_mut) = trans_state.form_state {
                                    form_mut.validation_error = Some(error);
                                }
                            }
                        }
                    } else {
                        // CREATE MODE - Build new transaction
                        match validators::validate_and_build_transaction(
                            form,
                            &trans_state.payees,
                            &trans_state.categories,
                            &date_format,
                        ) {
                            Ok(new_transaction) => {
                                // Spawn background task to create transaction if we have a budget ID
                                if let Some(budget_id) = budget_id_opt {
                                    let data_loader = data_loader.clone();
                                    let future = async move {
                                        data_loader
                                            .create_transaction(budget_id, new_transaction)
                                            .await;
                                    };
                                    task_manager
                                        .spawn_load_task("create_transaction".to_string(), future);
                                }
                            }
                            Err(error) => {
                                // Set validation error in form
                                if let Some(ref mut form_mut) = trans_state.form_state {
                                    form_mut.validation_error = Some(error);
                                }
                            }
                        }
                    }
                }
            }
        }

        AppCommand::LoadPayees { budget_id } => {
            let data_loader = data_loader.clone();
            let future = async move {
                data_loader.load_payees(budget_id, false).await;
            };
            task_manager.spawn_load_task("load_payees".to_string(), future);
        }

        AppCommand::LoadCategories { budget_id } => {
            let data_loader = data_loader.clone();
            let future = async move {
                data_loader.load_categories(budget_id, false).await;
            };
            task_manager.spawn_load_task("load_categories".to_string(), future);
        }

        AppCommand::ApproveTransaction {
            budget_id,
            transaction_id,
        } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                if let Some(ref mut transaction) = transactions_state
                    .transactions
                    .iter_mut()
                    .find(|t| t.id.to_string() == transaction_id)
                {
                    transaction.approved = true;

                    let transaction_id_clone = transaction_id.clone();
                    let api_client = data_loader.api_client.clone();
                    let data_tx = data_loader.data_tx.clone();
                    let budget_id_clone = budget_id.clone();
                    let future = async move {
                        let budget_id_api: BudgetId = budget_id_clone.clone().into();
                        let transaction_id: TransactionId = transaction_id_clone
                            .parse()
                            .expect("invalid transaction id");
                        let req = Request::transactions()
                            .with_budget(budget_id_api)
                            .update(transaction_id)
                            .approved(true);

                        match api_client.send(req).await {
                            Ok(_) => {
                                tracing::info!(
                                    "Transaction {} approved successfully on server",
                                    transaction_id_clone
                                );
                                let _ = data_tx.send(DataEvent::TransactionUpdated {
                                    transaction_id: transaction_id_clone,
                                });
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to approve transaction {}: {}",
                                    transaction_id_clone,
                                    e
                                );
                                let _ = data_tx.send(DataEvent::TransactionApproveFailed {
                                    transaction_id: transaction_id_clone,
                                    error: e.to_string(),
                                });
                            }
                        }
                    };

                    task_manager.spawn_load_task(
                        format!("approve_transaction_{}", transaction_id.clone()),
                        future,
                    );
                }
            }
        }

        AppCommand::InitiateTransactionDelete { transaction_id } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions_state.input_mode = InputMode::DeleteConfirmation;
                transactions_state.delete_confirmation_transaction_id = Some(transaction_id);
            }
        }

        AppCommand::ConfirmTransactionDelete {
            transaction_id,
            budget_id,
        } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                // 1. Optimistically remove from local state
                transactions_state
                    .transactions
                    .retain(|t| t.id.to_string() != transaction_id);

                // 2. Clear confirmation state and return to normal mode
                transactions_state.input_mode = InputMode::Normal;
                transactions_state.delete_confirmation_transaction_id = None;

                // 3. Reset table selection if needed
                let num_transactions = transactions_state.filtered_transactions().len();
                let mut table_state = transactions_state.table_state.borrow_mut();
                if let Some(selected) = table_state.selected() {
                    if selected >= num_transactions && num_transactions > 0 {
                        table_state.select(Some(num_transactions - 1));
                    } else if num_transactions == 0 {
                        table_state.select(None);
                    }
                }
                drop(table_state);

                // 4. Spawn background task to call DELETE API
                let api_client = data_loader.api_client.clone();
                let data_tx = data_loader.data_tx.clone();
                let transaction_id_clone = transaction_id.clone();
                let budget_id_clone = budget_id.clone();

                let future = async move {
                    let budget_id_api: BudgetId = budget_id_clone.into();
                    let transaction_id: TransactionId = transaction_id_clone
                        .parse()
                        .expect("invalid transaction id");
                    let req = Request::transactions()
                        .with_budget(budget_id_api)
                        .delete(transaction_id);

                    match api_client.send(req).await {
                        Ok(_) => {
                            tracing::info!(
                                "Successfully deleted transaction {}",
                                transaction_id_clone
                            );
                            let _ = data_tx.send(DataEvent::TransactionDeleted {
                                transaction_id: transaction_id_clone,
                            });
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to delete transaction {}: {}",
                                transaction_id_clone,
                                e
                            );
                            let _ = data_tx.send(DataEvent::TransactionDeleteFailed {
                                transaction_id: transaction_id_clone,
                                error: e.to_string(),
                            });
                        }
                    }
                };

                task_manager
                    .spawn_load_task(format!("delete_transaction_{}", transaction_id), future);
            }
        }

        AppCommand::CancelTransactionDelete => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions_state.input_mode = InputMode::Normal;
                transactions_state.delete_confirmation_transaction_id = None;
            }
        }

        AppCommand::InitiateTransactionEdit { transaction_id } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                // Find the transaction
                if let Some(transaction) = transactions_state
                    .transactions
                    .iter()
                    .find(|t| t.id.to_string() == transaction_id)
                {
                    // Check if reconciled - if so, show confirmation
                    if transaction.cleared == ReconciliationStatus::Reconciled {
                        transactions_state.input_mode = InputMode::ReconciledEditConfirmation;
                        transactions_state.reconciled_edit_transaction_id = Some(transaction_id);
                    } else {
                        // Proceed directly to edit - use recursive execute_command
                        execute_command(
                            AppCommand::EnterTransactionEditMode { transaction_id },
                            state,
                            task_manager,
                            data_loader,
                        );
                    }
                }
            }
        }

        AppCommand::ConfirmReconciledEdit { transaction_id } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions_state.input_mode = InputMode::Normal;
                transactions_state.reconciled_edit_transaction_id = None;

                // Proceed to edit mode - use recursive execute_command
                execute_command(
                    AppCommand::EnterTransactionEditMode { transaction_id },
                    state,
                    task_manager,
                    data_loader,
                );
            }
        }

        AppCommand::CancelReconciledEdit => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions_state.input_mode = InputMode::Normal;
                transactions_state.reconciled_edit_transaction_id = None;
            }
        }

        AppCommand::InitiateReconcile { cleared_balance } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions_state.input_mode = InputMode::ReconcileConfirmation;
                transactions_state.reconcile_cleared_balance = Some(cleared_balance);
            }
        }

        AppCommand::ConfirmReconcile {
            budget_id,
            account_id,
        } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                // Collect cleared transaction IDs and optimistically update them
                let transaction_ids: Vec<String> = transactions_state
                    .transactions
                    .iter()
                    .filter(|t| t.cleared == ReconciliationStatus::Cleared)
                    .map(|t| t.id.to_string())
                    .collect();

                // Optimistically update local state
                for transaction in transactions_state.transactions.iter_mut() {
                    if transaction.cleared == ReconciliationStatus::Cleared {
                        transaction.cleared = ReconciliationStatus::Reconciled;
                    }
                }

                // Clear confirmation state
                transactions_state.input_mode = InputMode::Normal;
                transactions_state.reconcile_cleared_balance = None;

                // Spawn background task to bulk update via API
                if !transaction_ids.is_empty() {
                    let api_client = data_loader.api_client.clone();
                    let data_tx = data_loader.data_tx.clone();
                    let cache = data_loader.cache.clone();
                    let transaction_ids_clone = transaction_ids.clone();
                    let budget_id_clone = budget_id.clone();
                    let account_id_clone = account_id.clone();

                    let future = async move {
                        let budget_id_api: BudgetId = budget_id_clone.clone().into();
                        let bulk_updates: Vec<BulkTransactionUpdate> = transaction_ids_clone
                            .iter()
                            .map(|id| BulkTransactionUpdate {
                                id: id.parse().expect("invalid transaction id"),
                                cleared: Some(ReconciliationStatus::Reconciled),
                            })
                            .collect();

                        let req = Request::transactions()
                            .bulk()
                            .update()
                            .budget_id(budget_id_api)
                            .transactions(bulk_updates);

                        match api_client.send(req).await {
                            Ok(_) => {
                                tracing::info!(
                                    "Successfully reconciled {} transactions",
                                    transaction_ids.len()
                                );
                                let _ = data_tx
                                    .send(DataEvent::TransactionsReconciled { transaction_ids });
                                // Invalidate cache so next load gets fresh data
                                let _ = cache
                                    .invalidate_transactions(&budget_id_clone, &account_id_clone)
                                    .await;
                            }
                            Err(e) => {
                                tracing::error!("Failed to reconcile transactions: {}", e);
                                let _ = data_tx.send(DataEvent::TransactionsReconcileFailed {
                                    error: e.to_string(),
                                });
                            }
                        }
                    };

                    task_manager.spawn_load_task("reconcile_transactions".to_string(), future);
                }
            }
        }

        AppCommand::CancelReconcile => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions_state.input_mode = InputMode::Normal;
                transactions_state.reconcile_cleared_balance = None;
            }
        }

        AppCommand::EnterTransactionEditMode { transaction_id } => {
            let budget_id_opt = state.current_budget_id.clone();
            let date_format = state
                .current_budget
                .as_ref()
                .and_then(|b| b.date_format.as_ref())
                .map(|d| d.format.clone())
                .unwrap_or_else(|| "YYYY-MM-DD".to_string());

            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                // Find the transaction
                if let Some(transaction) = trans_state
                    .transactions
                    .iter()
                    .find(|t| t.id.to_string() == transaction_id)
                {
                    trans_state.input_mode = InputMode::TransactionForm;
                    // Use from_transaction() constructor
                    trans_state.form_state = Some(TransactionFormState::from_transaction(
                        transaction,
                        &date_format,
                    ));

                    // Load payees/categories if not already loaded
                    if let Some(budget_id) = budget_id_opt {
                        if trans_state.payees.is_empty() {
                            let data_loader = data_loader.clone();
                            let budget_id_clone = budget_id.clone();
                            let future = async move {
                                data_loader.load_payees(budget_id_clone, false).await;
                            };
                            task_manager.spawn_load_task("load_payees".to_string(), future);
                        }

                        if trans_state.categories.is_empty() {
                            let data_loader = data_loader.clone();
                            let future = async move {
                                data_loader.load_categories(budget_id, false).await;
                            };
                            task_manager.spawn_load_task("load_categories".to_string(), future);
                        }
                    }
                }
            }
        }

        AppCommand::NavigateToLogs => {
            // Navigate to logs screen (no logging to avoid feedback loop)
            state.navigate_to(Screen::Logs(LogsState::default()));
        }

        AppCommand::ScrollLogsUp => {
            if let Screen::Logs(logs_state) = state.current_screen_mut() {
                // Scroll up means going back in time (increase offset)
                if logs_state.scroll_offset < logs_state.total_entries.saturating_sub(1) {
                    logs_state.scroll_offset += 1;
                }
            }
        }

        AppCommand::ScrollLogsDown => {
            if let Screen::Logs(logs_state) = state.current_screen_mut() {
                // Scroll down means going forward in time (decrease offset)
                logs_state.scroll_offset = logs_state.scroll_offset.saturating_sub(1);
            }
        }

        AppCommand::ScrollLogsPageUp => {
            if let Screen::Logs(logs_state) = state.current_screen_mut() {
                // Page up - scroll back 20 entries
                let page_size = 20;
                logs_state.scroll_offset = (logs_state.scroll_offset + page_size)
                    .min(logs_state.total_entries.saturating_sub(1));
            }
        }

        AppCommand::ScrollLogsPageDown => {
            if let Screen::Logs(logs_state) = state.current_screen_mut() {
                // Page down - scroll forward 20 entries
                let page_size = 20;
                logs_state.scroll_offset = logs_state.scroll_offset.saturating_sub(page_size);
            }
        }

        AppCommand::ScrollLogsToTop => {
            if let Screen::Logs(logs_state) = state.current_screen_mut() {
                logs_state.scroll_offset = logs_state.total_entries.saturating_sub(1);
            }
        }

        AppCommand::ScrollLogsToBottom => {
            if let Screen::Logs(logs_state) = state.current_screen_mut() {
                logs_state.scroll_offset = 0;
            }
        }

        AppCommand::NavigateBack => {
            // Navigate back in history (pop from navigation stack)
            state.navigate_back();
        }

        // Budget editing commands
        AppCommand::InitiateBudgetEdit { category_id } => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                // Find the category
                if let Some(category) = plan_state
                    .categories
                    .iter()
                    .find(|c| c.id.to_string() == category_id)
                {
                    plan_state.input_mode = InputMode::BudgetEdit;
                    plan_state.budget_form = Some(BudgetFormState::new(
                        category.id.to_string(),
                        category.name.clone(),
                        category.budgeted.into(),
                    ));
                }
            }
        }

        AppCommand::ExitBudgetEditMode => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                plan_state.input_mode = InputMode::Normal;
                plan_state.budget_form = None;
            }
        }

        AppCommand::AppendBudgetChar(c) => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                if let Some(ref mut form) = plan_state.budget_form {
                    form.budgeted_input.push(c);
                    form.validation_error = None;
                }
            }
        }

        AppCommand::DeleteBudgetChar => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                if let Some(ref mut form) = plan_state.budget_form {
                    form.budgeted_input.pop();
                    form.validation_error = None;
                }
            }
        }

        AppCommand::SubmitBudgetEdit { budget_id, month } => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                if let Some(ref mut form) = plan_state.budget_form {
                    // Evaluate math expression if present
                    let input = if let Some(result) =
                        utils::math::evaluate_expression(&form.budgeted_input)
                    {
                        result
                    } else {
                        form.budgeted_input.clone()
                    };

                    // Parse the amount
                    match input.parse::<f64>() {
                        Ok(amount) => {
                            use ynab_api::endpoints::Milliunits;
                            let budgeted_milliunits = (amount * 1000.0) as i64;
                            let category_id = form.category_id.clone();
                            let original_budgeted = form.original_budgeted;
                            let delta: Milliunits =
                                (budgeted_milliunits - original_budgeted).into();

                            // Optimistic update: apply locally immediately
                            if let Some(category) = plan_state
                                .categories
                                .iter_mut()
                                .find(|c| c.id.to_string() == category_id)
                            {
                                category.budgeted = budgeted_milliunits.into();
                            }

                            // Update month summary (budgeted increases, to_be_budgeted decreases)
                            if let Some(ref mut month_detail) = plan_state.month {
                                month_detail.budgeted = month_detail.budgeted + delta;
                                month_detail.to_be_budgeted = month_detail.to_be_budgeted - delta;
                            }

                            // Exit edit mode
                            plan_state.input_mode = InputMode::Normal;
                            plan_state.budget_form = None;

                            // Spawn background task to update via API
                            let data_loader = data_loader.clone();
                            let future = async move {
                                data_loader
                                    .update_category_budget(
                                        budget_id,
                                        month,
                                        category_id,
                                        budgeted_milliunits,
                                        original_budgeted,
                                    )
                                    .await;
                            };

                            task_manager.spawn_load_task("update_budget".to_string(), future);
                        }
                        Err(_) => {
                            form.validation_error =
                                Some("Invalid amount. Enter a number (e.g., 150.00)".to_string());
                        }
                    }
                }
            }
        }

        AppCommand::EnterSplitMode => {
            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                if let Some(ref mut form) = trans_state.form_state {
                    if !form.is_split_mode {
                        form.is_split_mode = true;
                        form.subtransactions.push(SubTransactionFormState::new());
                        form.category.clear();
                        form.filtered_categories.clear();
                        form.category_selection_index = 0;
                        // Focus the first subtransaction's amount field
                        //form.active_subtransaction_index = Some(0);
                        //form.subtransaction_field = SubTransactionField::Amount;
                    }
                }
            }
        }

        AppCommand::AddSubtransaction => {
            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                if let Some(ref mut form) = trans_state.form_state {
                    if form.is_split_mode {
                        // Add a new subtransaction
                        form.subtransactions.push(SubTransactionFormState::new());
                        // Focus the new subtransaction's amount field
                        let new_index = form.subtransactions.len() - 1;
                        form.active_subtransaction_index = Some(new_index);
                        form.subtransaction_field = SubTransactionField::Amount;
                    }
                }
            }
        }

        AppCommand::DeleteSubtransaction => {
            if let Screen::Transactions(trans_state) = state.current_screen_mut() {
                if let Some(ref mut form) = trans_state.form_state {
                    if form.is_split_mode {
                        if let Some(active_idx) = form.active_subtransaction_index {
                            if form.subtransactions.len() > 1 {
                                // Remove current subtransaction
                                form.subtransactions.remove(active_idx);
                                // Adjust focus index
                                if active_idx >= form.subtransactions.len() {
                                    form.active_subtransaction_index =
                                        Some(form.subtransactions.len() - 1);
                                }
                            } else {
                                // Only 1 subtransaction left - exit split mode
                                form.is_split_mode = false;
                                form.subtransactions.clear();
                                form.active_subtransaction_index = None;
                                form.current_field = Some(FormField::Category);
                            }
                        }
                    }
                }
            }
        }

        AppCommand::Quit => {
            state.should_quit = true;
        }
    }

    // Clear pending key after any command except SetPendingKey
    // This ensures multi-key sequences are properly reset after completion
    if !is_setting_pending_key && state.pending_key.is_some() {
        state.pending_key = None;
    }
}

/// Synchronous command execution for testing (no background tasks)
///
/// This function handles commands that only update state without spawning
/// background tasks. For commands that require API calls (LoadBudgets, LoadAccounts, etc.),
/// tests should inject DataEvents directly instead.
///
/// Only handles pure state transitions:
/// - UI state changes (help, pending keys, quit)
/// - Navigation (back, select next/prev, navigate to top/bottom)
/// - Filter mode (enter, exit, append/delete chars)
/// - View toggles (show deleted, show reconciled)
/// - Form mode transitions
///
/// NOTE: This is public for use by the testing module but should not be used in production code.
pub fn execute_command_sync(command: AppCommand, state: &mut AppState) {
    let is_setting_pending_key = matches!(command, AppCommand::SetPendingKey(_));

    match command {
        // Simple state updates
        AppCommand::Quit => state.should_quit = true,
        AppCommand::ToggleHelp => state.help_visible = !state.help_visible,
        AppCommand::SetPendingKey(c) => state.pending_key = Some(c),
        AppCommand::ClearPendingKey => state.pending_key = None,

        // Navigation
        AppCommand::NavigateBack => {
            state.navigate_back();
        }
        AppCommand::NavigateToTop => match state.current_screen_mut() {
            Screen::Budgets(s) => s.selected_budget_index = 0,
            Screen::Accounts(s) => s.table_state.borrow_mut().select(Some(0)),
            Screen::Transactions(s) => s.table_state.borrow_mut().select(Some(0)),
            Screen::Plan(s) => s.table_state.borrow_mut().select(Some(0)),
            Screen::Logs(s) => s.scroll_offset = s.total_entries.saturating_sub(1),
        },
        AppCommand::NavigateToBottom => match state.current_screen_mut() {
            Screen::Budgets(s) => {
                if !s.budgets.is_empty() {
                    s.selected_budget_index = s.budgets.len() - 1;
                }
            }
            Screen::Accounts(s) => {
                let len = s.filtered_accounts().len();
                if len > 0 {
                    s.table_state.borrow_mut().select(Some(len - 1));
                }
            }
            Screen::Transactions(s) => {
                let len = s.filtered_transactions().len();
                if len > 0 {
                    s.table_state.borrow_mut().select(Some(len - 1));
                }
            }
            Screen::Plan(s) => {
                let len = s.filtered_categories().len();
                if len > 0 {
                    s.table_state.borrow_mut().select(Some(len - 1));
                }
            }
            Screen::Logs(s) => s.scroll_offset = 0,
        },
        AppCommand::SelectNext => match state.current_screen_mut() {
            Screen::Budgets(s) => {
                if !s.budgets.is_empty() {
                    s.selected_budget_index = (s.selected_budget_index + 1) % s.budgets.len();
                }
            }
            Screen::Accounts(s) => s.select_next(),
            Screen::Transactions(s) => s.select_next(),
            Screen::Plan(s) => s.select_next(),
            Screen::Logs(_) => {} // Uses scroll commands instead
        },
        AppCommand::SelectPrevious => match state.current_screen_mut() {
            Screen::Budgets(s) => {
                if !s.budgets.is_empty() {
                    if s.selected_budget_index == 0 {
                        s.selected_budget_index = s.budgets.len() - 1;
                    } else {
                        s.selected_budget_index -= 1;
                    }
                }
            }
            Screen::Accounts(s) => s.select_prev(),
            Screen::Transactions(s) => s.select_prev(),
            Screen::Plan(s) => s.select_prev(),
            Screen::Logs(_) => {} // Uses scroll commands instead
        },

        // Filter mode
        AppCommand::EnterFilterMode => match state.current_screen_mut() {
            Screen::Accounts(s) => s.input_mode = InputMode::Filter,
            Screen::Transactions(s) => s.input_mode = InputMode::Filter,
            _ => {}
        },
        AppCommand::ExitFilterMode => match state.current_screen_mut() {
            Screen::Accounts(s) => s.input_mode = InputMode::Normal,
            Screen::Transactions(s) => s.input_mode = InputMode::Normal,
            _ => {}
        },
        AppCommand::AppendFilterChar(c) => match state.current_screen_mut() {
            Screen::Accounts(s) => s.filter_query.push(c),
            Screen::Transactions(s) => s.filter_query.push(c),
            _ => {}
        },
        AppCommand::DeleteFilterChar => match state.current_screen_mut() {
            Screen::Accounts(s) => {
                s.filter_query.pop();
            }
            Screen::Transactions(s) => {
                s.filter_query.pop();
            }
            _ => {}
        },
        AppCommand::ClearFilter => match state.current_screen_mut() {
            Screen::Accounts(s) => {
                s.filter_query.clear();
                s.input_mode = InputMode::Normal;
            }
            Screen::Transactions(s) => {
                s.filter_query.clear();
                s.input_mode = InputMode::Normal;
            }
            _ => {}
        },

        // View toggles
        AppCommand::ToggleShowClosedAccounts => {
            if let Screen::Accounts(s) = state.current_screen_mut() {
                s.show_closed_accounts = !s.show_closed_accounts;
            }
        }
        AppCommand::ToggleShowReconciledTransactions => {
            if let Screen::Transactions(s) = state.current_screen_mut() {
                s.show_reconciled_transactions = !s.show_reconciled_transactions;
            }
        }
        AppCommand::TogglePlanFocusedView => {
            if let Screen::Plan(s) = state.current_screen_mut() {
                s.focused_view = s.focused_view.next();
                s.table_state = RefCell::new(TableState::default().with_selected(0));
            }
        }

        AppCommand::ExitTransactionCreateMode => {
            if let Screen::Transactions(s) = state.current_screen_mut() {
                s.input_mode = InputMode::Normal;
                s.form_state = None;
            }
        }

        // Log screen commands - can be handled synchronously
        AppCommand::NavigateToLogs => {
            state.navigate_to(Screen::Logs(LogsState::default()));
        }
        AppCommand::ScrollLogsUp => {
            if let Screen::Logs(s) = state.current_screen_mut() {
                if s.scroll_offset < s.total_entries.saturating_sub(1) {
                    s.scroll_offset += 1;
                }
            }
        }
        AppCommand::ScrollLogsDown => {
            if let Screen::Logs(s) = state.current_screen_mut() {
                s.scroll_offset = s.scroll_offset.saturating_sub(1);
            }
        }
        AppCommand::ScrollLogsPageUp => {
            if let Screen::Logs(s) = state.current_screen_mut() {
                s.scroll_offset = (s.scroll_offset + 20).min(s.total_entries.saturating_sub(1));
            }
        }
        AppCommand::ScrollLogsPageDown => {
            if let Screen::Logs(s) = state.current_screen_mut() {
                s.scroll_offset = s.scroll_offset.saturating_sub(20);
            }
        }
        AppCommand::ScrollLogsToTop => {
            if let Screen::Logs(s) = state.current_screen_mut() {
                s.scroll_offset = s.total_entries.saturating_sub(1);
            }
        }
        AppCommand::ScrollLogsToBottom => {
            if let Screen::Logs(s) = state.current_screen_mut() {
                s.scroll_offset = 0;
            }
        }

        // Budget edit mode (sync state changes only)
        AppCommand::InitiateBudgetEdit { category_id } => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                if let Some(category) = plan_state
                    .categories
                    .iter()
                    .find(|c| c.id.to_string() == category_id)
                {
                    plan_state.input_mode = InputMode::BudgetEdit;
                    plan_state.budget_form = Some(BudgetFormState::new(
                        category.id.to_string(),
                        category.name.clone(),
                        category.budgeted.into(),
                    ));
                }
            }
        }

        AppCommand::ExitBudgetEditMode => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                plan_state.input_mode = InputMode::Normal;
                plan_state.budget_form = None;
            }
        }

        AppCommand::AppendBudgetChar(c) => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                if let Some(ref mut form) = plan_state.budget_form {
                    form.budgeted_input.push(c);
                    form.validation_error = None;
                }
            }
        }

        AppCommand::DeleteBudgetChar => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                if let Some(ref mut form) = plan_state.budget_form {
                    form.budgeted_input.pop();
                    form.validation_error = None;
                }
            }
        }

        // Commands that require background tasks - skip in sync mode
        // Tests should inject DataEvents directly for these
        AppCommand::LoadBudgets { .. }
        | AppCommand::LoadAccounts { .. }
        | AppCommand::LoadTransactions { .. }
        | AppCommand::LoadPlan { .. }
        | AppCommand::LoadPlanMonth { .. }
        | AppCommand::NavigatePlanMonth { .. }
        | AppCommand::LoadPayees { .. }
        | AppCommand::LoadCategories { .. }
        | AppCommand::ToggleTransactionCleared { .. }
        | AppCommand::EnterTransactionCreateMode
        | AppCommand::NavigateFormField { .. }
        | AppCommand::AppendFormFieldChar { .. }
        | AppCommand::DeleteFormFieldChar
        | AppCommand::ClearFormField
        | AppCommand::SelectAutocompleteItem { .. }
        | AppCommand::ConfirmAutocompleteSelection
        | AppCommand::SubmitTransactionForm
        | AppCommand::EnterSplitMode
        | AppCommand::AddSubtransaction
        | AppCommand::DeleteSubtransaction
        | AppCommand::ApproveTransaction { .. }
        | AppCommand::InitiateTransactionDelete { .. }
        | AppCommand::ConfirmTransactionDelete { .. }
        | AppCommand::CancelTransactionDelete
        | AppCommand::InitiateTransactionEdit { .. }
        | AppCommand::ConfirmReconciledEdit { .. }
        | AppCommand::CancelReconciledEdit
        | AppCommand::EnterTransactionEditMode { .. }
        | AppCommand::InitiateReconcile { .. }
        | AppCommand::ConfirmReconcile { .. }
        | AppCommand::CancelReconcile
        | AppCommand::SubmitBudgetEdit { .. } => {
            // Skip - tests will inject corresponding DataEvents
        }
    }

    // Clear pending key after any command except SetPendingKey
    if !is_setting_pending_key && state.pending_key.is_some() {
        state.pending_key = None;
    }
}

/// Compute the adjacent month (next or previous) from a given month string
/// Input format: YYYY-MM-DD (first day of month)
/// Returns the first day of the next/previous month in the same format
fn compute_adjacent_month(current_month: &str, forward: bool) -> Option<String> {
    use chrono::{Datelike, NaiveDate};

    // Parse the date
    let date = NaiveDate::parse_from_str(current_month, "%Y-%m-%d").ok()?;

    // Calculate new month
    let new_date = if forward {
        // Go to next month
        if date.month() == 12 {
            NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)?
        } else {
            NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)?
        }
    } else {
        // Go to previous month
        if date.month() == 1 {
            NaiveDate::from_ymd_opt(date.year() - 1, 12, 1)?
        } else {
            NaiveDate::from_ymd_opt(date.year(), date.month() - 1, 1)?
        }
    };

    Some(new_date.format("%Y-%m-%d").to_string())
}
