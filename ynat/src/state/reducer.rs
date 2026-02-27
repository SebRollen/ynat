use super::{autocomplete, AppState, InputMode, LoadingState};
use crate::events::DataEvent;
use crate::ui::screens::Screen;
use ratatui::widgets::TableState;
use std::cell::RefCell;
use ynab_api::endpoints::{
    accounts::{Account, AccountType},
    transactions::Transaction,
};

/// Pure state transition function for data events
pub fn reduce_data_event(state: &mut AppState, event: DataEvent) {
    match event {
        // Budgets cache loaded
        DataEvent::BudgetsCacheLoaded {
            budgets,
            default_budget,
        } => {
            if state.current_budget.is_none() {
                state.current_budget = default_budget;
            }
            if let Screen::Budgets(budgets_state) = state.current_screen_mut() {
                budgets_state.budgets = budgets;
                budgets_state.budgets_loading = LoadingState::Loaded;
                budgets_state.selected_budget_index = 0;
            }
        }

        // Budgets loaded from API
        DataEvent::BudgetsLoaded {
            budgets,
            default_budget,
        } => {
            if state.current_budget.is_none() {
                state.current_budget = default_budget;
            }
            if let Screen::Budgets(budgets_state) = state.current_screen_mut() {
                budgets_state.budgets = budgets;
                budgets_state.budgets_loading = LoadingState::Loaded;
            }
        }

        // Accounts cache loaded
        DataEvent::AccountsCacheLoaded { mut accounts } => match state.current_screen_mut() {
            Screen::Accounts(accounts_state) => {
                accounts.sort_by_key(|account| account_type_sort_order(account.account_type));
                accounts_state.accounts = accounts;
                accounts_state.accounts_loading = LoadingState::Loaded;
                accounts_state.table_state = RefCell::new(TableState::default().with_selected(0));
            }
            Screen::Transactions(transactions_state) => {
                transactions_state.accounts = accounts;
            }
            _ => {}
        },

        // Accounts loaded from API
        DataEvent::AccountsLoaded { mut accounts } => match state.current_screen_mut() {
            Screen::Accounts(accounts_state) => {
                accounts.sort_by_key(|account| account_type_sort_order(account.account_type));
                accounts_state.accounts = accounts;
                accounts_state.accounts_loading = LoadingState::Loaded;
            }
            Screen::Transactions(transactions_state) => {
                transactions_state.accounts = accounts;
            }
            _ => {}
        },

        // Accounts delta loaded (merge into existing)
        DataEvent::AccountsDeltaLoaded { delta } => match state.current_screen_mut() {
            Screen::Accounts(accounts_state) => {
                merge_accounts_delta(&mut accounts_state.accounts, delta);
                accounts_state.accounts_loading = LoadingState::Loaded;
            }
            Screen::Transactions(transactions_state) => {
                merge_accounts_delta(&mut transactions_state.accounts, delta);
            }
            _ => {}
        },

        // Transactions cache loaded
        DataEvent::TransactionsCacheLoaded { mut transactions } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                // Sort in descending date order (most recent first)
                transactions.sort_by(|a, b| b.date.cmp(&a.date));
                transactions_state.transactions = transactions;
                transactions_state.transactions_loading = LoadingState::Loaded;
                transactions_state.table_state =
                    RefCell::new(TableState::default().with_selected(0))
            }
        }

        // Transactions loaded from API
        DataEvent::TransactionsLoaded { mut transactions } => {
            // Sort in descending date order (most recent first)
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions.sort_by(|a, b| b.date.cmp(&a.date));
                transactions_state.transactions = transactions;
                transactions_state.transactions_loading = LoadingState::Loaded;
            }
        }

        // Transactions delta loaded (merge into existing)
        DataEvent::TransactionsDeltaLoaded { delta } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                merge_transactions_delta(&mut transactions_state.transactions, delta);
                transactions_state.transactions_loading = LoadingState::Loaded;
            }
        }

        // Plan cache loaded
        DataEvent::PlanCacheLoaded { month, categories } => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                plan_state.month = Some(month);
                plan_state.categories = categories;
                plan_state.plan_loading = LoadingState::Loaded;
                plan_state.table_state = RefCell::new(TableState::default().with_selected(0));
            }
        }

        // Plan data loaded
        DataEvent::PlanLoaded { month, categories } => {
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                plan_state.month = Some(month);
                plan_state.categories = categories;
                plan_state.plan_loading = LoadingState::Loaded;
            }
        }

        // Transaction updated successfully
        DataEvent::TransactionUpdated { transaction_id } => {
            // Optimistic update already applied, nothing to do
            tracing::debug!("Transaction {transaction_id} update confirmed by server");
        }

        // Transaction update failed - rollback optimistic update
        DataEvent::TransactionUpdateFailed {
            transaction_id,
            original_status,
            original_approved,
            error,
        } => {
            tracing::warn!("Rolling back transaction update: {}", error);
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                if let Some(transaction) = transactions_state
                    .transactions
                    .iter_mut()
                    .find(|t| t.id.to_string() == transaction_id)
                {
                    transaction.cleared = original_status;
                    transaction.approved = original_approved;
                    tracing::info!(
                        "Rolled back transaction {} to status: {}",
                        transaction_id,
                        transaction.cleared
                    );
                }
            }
        }

        // Payees loaded (for transaction creation)
        DataEvent::PayeesLoaded { payees } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions_state.payees = payees;

                // Update filtered payees in form if form is open
                if let Some(ref mut form) = transactions_state.form_state {
                    form.filtered_payees =
                        autocomplete::filter_payees(&transactions_state.payees, &form.payee);
                }
            }
        }

        // Categories loaded (for transaction creation)
        DataEvent::CategoriesLoaded { categories } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                transactions_state.categories = categories;

                // Update filtered categories in form if form is open
                if let Some(ref mut form) = transactions_state.form_state {
                    form.filtered_categories = autocomplete::filter_categories(
                        &transactions_state.categories,
                        &form.category,
                    );
                }
            }
        }

        DataEvent::TransactionApproved { .. } => {
            // Already locally approved, nothing to do
        }

        DataEvent::TransactionApproveFailed {
            transaction_id,
            error,
        } => {
            tracing::warn!("Rolling back transaction approval: {}", error);
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                if let Some(transaction) = transactions_state
                    .transactions
                    .iter_mut()
                    .find(|t| t.id.to_string() == transaction_id)
                {
                    transaction.approved = false;
                    tracing::info!("Rolled back transaction {} to unapproved", transaction_id,);
                }
            }
        }

        // Transaction created successfully
        DataEvent::TransactionCreated { transaction } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                // Add new transaction to list (at the beginning after sorting)
                transactions_state.transactions.push(transaction);
                transactions_state
                    .transactions
                    .sort_by(|a, b| b.date.cmp(&a.date));

                // Close form
                transactions_state.input_mode = InputMode::Normal;
                transactions_state.form_state = None;

                // Reset table selection to first item
                transactions_state.table_state =
                    RefCell::new(TableState::default().with_selected(0));

                tracing::info!("Transaction created successfully and added to list");
            }
        }

        // Transaction creation failed
        DataEvent::TransactionCreateFailed { error } => {
            tracing::error!("Transaction creation failed: {}", error);
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                // Keep form open, show error
                if let Some(ref mut form) = transactions_state.form_state {
                    form.validation_error = Some(error);
                }
            }
        }

        // Transaction deletion confirmed by API
        DataEvent::TransactionDeleted { transaction_id } => {
            // Optimistic removal already done, just log confirmation
            tracing::debug!(
                "Transaction {} deletion confirmed by server",
                transaction_id
            );
        }

        // Transaction deletion failed
        DataEvent::TransactionDeleteFailed {
            transaction_id,
            error,
        } => {
            tracing::error!("Failed to delete transaction {}: {}", transaction_id, error);
            // Transaction was already removed optimistically
            // User can manually refresh with 'r' key to reload if needed
            tracing::warn!("Transaction deletion failed. User should refresh with 'r' key.");
        }

        // Transaction edited (full update) confirmed by API
        DataEvent::TransactionUpdatedFull { transaction } => {
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                // Find and replace the transaction in the list
                if let Some(idx) = transactions_state
                    .transactions
                    .iter()
                    .position(|t| t.id == transaction.id)
                {
                    transactions_state.transactions[idx] = transaction;

                    // Re-sort by date (transactions may have moved if date changed)
                    transactions_state
                        .transactions
                        .sort_by(|a, b| b.date.cmp(&a.date));
                }

                // Close form
                transactions_state.input_mode = InputMode::Normal;
                transactions_state.form_state = None;

                tracing::info!("Transaction updated and form closed");
            }
        }

        // Transaction update failed
        DataEvent::TransactionUpdateFullFailed {
            transaction_id,
            error,
        } => {
            tracing::error!("Failed to update transaction {}: {}", transaction_id, error);
            if let Screen::Transactions(transactions_state) = state.current_screen_mut() {
                // Keep form open, show error
                if let Some(ref mut form) = transactions_state.form_state {
                    form.validation_error = Some(error);
                }
            }
        }

        // Transactions reconciled successfully
        DataEvent::TransactionsReconciled { transaction_ids } => {
            // Optimistic update already applied, just log confirmation
            tracing::info!(
                "{} transactions reconciled successfully",
                transaction_ids.len()
            );
        }

        // Transactions reconciliation failed - no rollback since optimistic update already applied
        DataEvent::TransactionsReconcileFailed { error } => {
            // We don't rollback here because the optimistic update is already applied.
            // User can manually refresh with 'r' key to reload if needed.
            tracing::error!(
                "Reconciliation failed: {}. User should refresh with 'r' key.",
                error
            );
        }

        // Load error
        DataEvent::LoadError { error } => {
            // Set error state for whichever resource was loading
            match state.current_screen_mut() {
                Screen::Accounts(accounts_state) => {
                    if matches!(accounts_state.accounts_loading, LoadingState::Loading(..)) {
                        accounts_state.accounts_loading = LoadingState::Error(error);
                    }
                }
                Screen::Transactions(transactions_state) => {
                    if matches!(
                        transactions_state.transactions_loading,
                        LoadingState::Loading(..)
                    ) {
                        transactions_state.transactions_loading = LoadingState::Error(error);
                    }
                }
                Screen::Budgets(budgets_state) => {
                    if matches!(budgets_state.budgets_loading, LoadingState::Loading(..)) {
                        budgets_state.budgets_loading = LoadingState::Error(error);
                    }
                }
                Screen::Plan(plan_state) => {
                    if matches!(plan_state.plan_loading, LoadingState::Loading(..)) {
                        plan_state.plan_loading = LoadingState::Error(error);
                    }
                }
                Screen::Logs(_) => {
                    // Logs screen has no loading state - ignore errors
                }
            }
        }

        // Budget category updates
        DataEvent::CategoryBudgetUpdated { category } => {
            tracing::info!(
                "Category {} budget updated to {}",
                category.id,
                category.budgeted
            );
            // Update the category in the plan state if we're on the plan screen
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                if let Some(existing) = plan_state
                    .categories
                    .iter_mut()
                    .find(|c| c.id == category.id)
                {
                    *existing = category;
                }
            }
        }

        DataEvent::CategoryBudgetUpdateFailed {
            category_id,
            original_budgeted,
            new_budgeted,
            error,
        } => {
            tracing::error!(
                "Failed to update category {} budget: {}. Rolling back to {}",
                category_id,
                error,
                original_budgeted
            );
            // Rollback the optimistic update
            if let Screen::Plan(plan_state) = state.current_screen_mut() {
                use ynab_api::endpoints::Milliunits;
                // Rollback category budgeted amount
                if let Some(category) = plan_state
                    .categories
                    .iter_mut()
                    .find(|c| c.id.to_string() == category_id)
                {
                    category.budgeted = original_budgeted.into();
                }

                // Rollback month summary (reverse the delta that was applied)
                let delta: Milliunits = (new_budgeted - original_budgeted).into();
                if let Some(ref mut month_detail) = plan_state.month {
                    month_detail.budgeted = month_detail.budgeted - delta;
                    month_detail.to_be_budgeted = month_detail.to_be_budgeted + delta;
                }
            }
        }
    }
}

/// Merge accounts delta into existing accounts list
fn merge_accounts_delta(accounts: &mut Vec<Account>, delta: Vec<Account>) {
    for delta_account in delta {
        if delta_account.deleted {
            // Remove deleted accounts
            accounts.retain(|a| a.id != delta_account.id);
        } else if let Some(existing) = accounts.iter_mut().find(|a| a.id == delta_account.id) {
            // Update existing account
            *existing = delta_account;
        } else {
            // Add new account
            accounts.push(delta_account);
        }
    }

    // Sort accounts by type after merge (to maintain consistent ordering)
    accounts.sort_by_key(|account| account_type_sort_order(account.account_type));
}

/// Merge transactions delta into existing transactions list
fn merge_transactions_delta(transactions: &mut Vec<Transaction>, delta: Vec<Transaction>) {
    for delta_transaction in delta {
        if delta_transaction.deleted {
            // Remove deleted transactions
            transactions.retain(|t| t.id != delta_transaction.id);
        } else if let Some(existing) = transactions
            .iter_mut()
            .find(|t| t.id == delta_transaction.id)
        {
            // Update existing transaction
            *existing = delta_transaction;
        } else {
            // Add new transaction
            transactions.push(delta_transaction);
        }
    }

    // Sort in descending date order (most recent first)
    transactions.sort_by(|a, b| b.date.cmp(&a.date));
}

/// Helper function to determine account type sort order
fn account_type_sort_order(account_type: AccountType) -> usize {
    use AccountType::*;
    match account_type {
        Checking | Savings | Cash => 0,
        CreditCard | LineOfCredit => 1,
        Mortgage | AutoLoan | StudentLoan | PersonalLoan | MedicalDebt | OtherDebt => 2,
        OtherAsset | OtherLiability => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AccountsState, BudgetsState, TransactionFormState, TransactionsState};
    use chrono::NaiveDate;
    use uuid::Uuid;
    use ynab_api::endpoints::{
        accounts::{Account, AccountType},
        budgets::BudgetSummary,
        categories::Category,
        payees::Payee,
        transactions::{ReconciliationStatus, Transaction},
        BudgetId, Milliunits, TransactionId,
    };

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Generate a deterministic UUID from a string ID for testing
    /// Uses a simple hash-based approach to create reproducible UUIDs
    fn test_uuid(id: &str) -> Uuid {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        id.hash(&mut hasher);
        let hash = hasher.finish();
        // Create a UUID from the hash bytes (padded/repeated as needed)
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
        Uuid::from_bytes(bytes)
    }

    /// Generate a TransactionId string from a test id (for event matching)
    fn test_transaction_id_str(id: &str) -> String {
        test_uuid(id).to_string()
    }

    fn create_test_account(id: &str, name: &str, account_type: AccountType) -> Account {
        Account {
            id: test_uuid(id),
            name: name.to_string(),
            account_type,
            on_budget: true,
            closed: false,
            note: None,
            balance: Milliunits::new(100000),
            cleared_balance: Milliunits::new(50000),
            uncleared_balance: Milliunits::new(50000),
            transfer_payee_id: None,
            direct_import_linked: false,
            direct_import_in_error: false,
            deleted: false,
        }
    }

    fn create_test_transaction(
        id: &str,
        date: &str,
        amount: i64,
        cleared: ReconciliationStatus,
    ) -> Transaction {
        Transaction {
            id: TransactionId::new(test_uuid(id)),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            amount: Milliunits::new(amount),
            memo: None,
            cleared,
            approved: true,
            flag_color: None,
            account_id: test_uuid("test_account"),
            account_name: "Checking".to_string(),
            payee_id: None,
            payee_name: None,
            category_id: None,
            category_name: None,
            transfer_account_id: None,
            transfer_transaction_id: None,
            matched_transaction_id: None,
            import_id: None,
            deleted: false,
            subtransactions: vec![],
        }
    }

    fn create_test_budget(id: &str, name: &str) -> BudgetSummary {
        BudgetSummary {
            id: test_uuid(id).into(),
            name: name.to_string(),
            last_modified_on: None,
            first_month: None,
            last_month: None,
            date_format: None,
            currency_format: None,
            accounts: None,
        }
    }

    // ============================================================================
    // Budgets Tests
    // ============================================================================

    #[test]
    fn test_budgets_cache_loaded() {
        let mut state = AppState::new();
        state.history = vec![Screen::Budgets(BudgetsState::default())];

        let budgets = vec![
            create_test_budget("b1", "Budget 1"),
            create_test_budget("b2", "Budget 2"),
        ];

        reduce_data_event(
            &mut state,
            DataEvent::BudgetsCacheLoaded {
                budgets: budgets.clone(),
                default_budget: Some(budgets[0].clone()),
            },
        );

        let Screen::Budgets(budgets_state) = state.current_screen() else {
            panic!("Expected Budgets screen");
        };
        assert_eq!(budgets_state.budgets.len(), 2);
        assert_eq!(budgets_state.budgets[0].id, BudgetId::from(test_uuid("b1")));
        assert_eq!(budgets_state.budgets_loading, LoadingState::Loaded);
        assert_eq!(budgets_state.selected_budget_index, 0);
    }

    #[test]
    fn test_budgets_loaded() {
        let mut state = AppState::new();
        state.history = vec![Screen::Budgets(BudgetsState::default())];

        let budgets = vec![create_test_budget("b1", "Budget 1")];

        reduce_data_event(
            &mut state,
            DataEvent::BudgetsLoaded {
                budgets: budgets.clone(),
                default_budget: Some(budgets[0].clone()),
            },
        );

        let Screen::Budgets(budgets_state) = state.current_screen() else {
            panic!("Expected Budgets screen");
        };
        assert_eq!(budgets_state.budgets.len(), 1);
        assert_eq!(budgets_state.budgets_loading, LoadingState::Loaded);
    }

    // ============================================================================
    // Accounts Tests
    // ============================================================================

    #[test]
    fn test_accounts_cache_loaded() {
        let mut state = AppState::new();
        state.history = vec![Screen::Accounts(AccountsState::default())];

        let accounts = vec![
            create_test_account("a1", "Checking", AccountType::Checking),
            create_test_account("a2", "Savings", AccountType::Savings),
        ];

        reduce_data_event(&mut state, DataEvent::AccountsCacheLoaded { accounts });

        let Screen::Accounts(accounts_state) = state.current_screen() else {
            panic!("Expected Accounts screen");
        };
        assert_eq!(accounts_state.accounts.len(), 2);
        assert_eq!(accounts_state.accounts_loading, LoadingState::Loaded);
        // Should have selection set to 0
        assert_eq!(accounts_state.table_state.borrow().selected(), Some(0));
    }

    #[test]
    fn test_accounts_sorted_by_type() {
        let mut state = AppState::new();
        state.history = vec![Screen::Accounts(AccountsState::default())];

        // Create accounts in wrong order
        let accounts = vec![
            create_test_account("a1", "Credit Card", AccountType::CreditCard),
            create_test_account("a2", "Cash", AccountType::Cash),
            create_test_account("a3", "Checking", AccountType::Checking),
        ];

        reduce_data_event(&mut state, DataEvent::AccountsCacheLoaded { accounts });

        let Screen::Accounts(accounts_state) = state.current_screen() else {
            panic!("Expected Accounts screen");
        };
        // Should be sorted: cash, checking, creditCard
        assert_eq!(accounts_state.accounts[0].account_type, AccountType::Cash);
        assert_eq!(
            accounts_state.accounts[1].account_type,
            AccountType::Checking
        );
        assert_eq!(
            accounts_state.accounts[2].account_type,
            AccountType::CreditCard
        );
    }

    #[test]
    fn test_accounts_delta_merge_update() {
        let mut state = AppState::new();
        state.history = vec![Screen::Accounts(AccountsState {
            accounts: vec![create_test_account("a1", "Checking", AccountType::Checking)],
            ..Default::default()
        })];

        // Delta with updated account
        let mut updated_account =
            create_test_account("a1", "Checking Updated", AccountType::Checking);
        updated_account.balance = 200000.into();

        reduce_data_event(
            &mut state,
            DataEvent::AccountsDeltaLoaded {
                delta: vec![updated_account],
            },
        );

        let Screen::Accounts(accounts_state) = state.current_screen() else {
            panic!("Expected Accounts screen");
        };
        assert_eq!(accounts_state.accounts.len(), 1);
        assert_eq!(accounts_state.accounts[0].name, "Checking Updated");
        assert_eq!(accounts_state.accounts[0].balance, Milliunits::new(200000));
    }

    #[test]
    fn test_accounts_delta_merge_add() {
        let mut state = AppState::new();
        state.history = vec![Screen::Accounts(AccountsState {
            accounts: vec![create_test_account("a1", "Checking", AccountType::Checking)],
            ..Default::default()
        })];

        // Delta with new account
        let new_account = create_test_account("a2", "Savings", AccountType::Savings);

        reduce_data_event(
            &mut state,
            DataEvent::AccountsDeltaLoaded {
                delta: vec![new_account],
            },
        );

        let Screen::Accounts(accounts_state) = state.current_screen() else {
            panic!("Expected Accounts screen");
        };
        assert_eq!(accounts_state.accounts.len(), 2);
    }

    #[test]
    fn test_accounts_delta_merge_delete() {
        let mut state = AppState::new();
        state.history = vec![Screen::Accounts(AccountsState {
            accounts: vec![
                create_test_account("a1", "Checking", AccountType::Checking),
                create_test_account("a2", "Savings", AccountType::Savings),
            ],
            ..Default::default()
        })];

        // Delta with deleted account
        let mut deleted_account = create_test_account("a1", "Checking", AccountType::Checking);
        deleted_account.deleted = true;

        reduce_data_event(
            &mut state,
            DataEvent::AccountsDeltaLoaded {
                delta: vec![deleted_account],
            },
        );

        let Screen::Accounts(accounts_state) = state.current_screen() else {
            panic!("Expected Accounts screen");
        };
        assert_eq!(accounts_state.accounts.len(), 1);
        assert_eq!(accounts_state.accounts[0].name, "Savings"); // a2 was "Savings"
    }

    // ============================================================================
    // Transactions Tests
    // ============================================================================

    #[test]
    fn test_transactions_cache_loaded() {
        let mut state = AppState::new();
        state.history = vec![Screen::Transactions(Box::default())];

        let transactions = vec![
            create_test_transaction("t1", "2024-01-15", -5000, ReconciliationStatus::Cleared),
            create_test_transaction("t2", "2024-01-10", -3000, ReconciliationStatus::Uncleared),
        ];

        reduce_data_event(
            &mut state,
            DataEvent::TransactionsCacheLoaded { transactions },
        );

        let Screen::Transactions(trans_state) = state.current_screen() else {
            panic!("Expected Transactions screen");
        };
        assert_eq!(trans_state.transactions.len(), 2);
        assert_eq!(trans_state.transactions_loading, LoadingState::Loaded);
        // Should be sorted by date descending (most recent first)
        assert_eq!(
            trans_state.transactions[0].date,
            NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap()
        );
        assert_eq!(
            trans_state.transactions[1].date,
            NaiveDate::parse_from_str("2024-01-10", "%Y-%m-%d").unwrap()
        );
    }

    #[test]
    fn test_transactions_sorted_by_date_descending() {
        let mut state = AppState::new();
        state.history = vec![Screen::Transactions(Box::default())];

        // Create transactions in wrong order
        let transactions = vec![
            create_test_transaction("t1", "2024-01-10", -5000, ReconciliationStatus::Cleared),
            create_test_transaction("t2", "2024-01-20", -3000, ReconciliationStatus::Uncleared),
            create_test_transaction("t3", "2024-01-15", -2000, ReconciliationStatus::Cleared),
        ];

        reduce_data_event(
            &mut state,
            DataEvent::TransactionsCacheLoaded { transactions },
        );

        let Screen::Transactions(trans_state) = state.current_screen() else {
            panic!("Expected Transactions screen");
        };
        // Should be sorted newest first
        assert_eq!(
            trans_state.transactions[0].date,
            NaiveDate::parse_from_str("2024-01-20", "%Y-%m-%d").unwrap()
        );
        assert_eq!(
            trans_state.transactions[1].date,
            NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap()
        );
        assert_eq!(
            trans_state.transactions[2].date,
            NaiveDate::parse_from_str("2024-01-10", "%Y-%m-%d").unwrap()
        );
    }

    #[test]
    fn test_transaction_update_failed_rollback() {
        let mut state = AppState::new();
        state.history = vec![Screen::Transactions(Box::new(TransactionsState {
            transactions: vec![create_test_transaction(
                "t1",
                "2024-01-15",
                -5000,
                ReconciliationStatus::Cleared,
            )],
            ..Default::default()
        }))];

        // Simulate failed update - should rollback to original status
        reduce_data_event(
            &mut state,
            DataEvent::TransactionUpdateFailed {
                transaction_id: test_transaction_id_str("t1"),
                original_status: ReconciliationStatus::Uncleared,
                original_approved: false,
                error: "API error".to_string(),
            },
        );

        let Screen::Transactions(trans_state) = state.current_screen() else {
            panic!("Expected Transactions screen");
        };
        assert_eq!(
            trans_state.transactions[0].cleared,
            ReconciliationStatus::Uncleared
        );
        assert!(!trans_state.transactions[0].approved);
    }

    #[test]
    fn test_transaction_created() {
        let mut state = AppState::new();
        state.history = vec![Screen::Transactions(Box::new(TransactionsState {
            transactions: vec![create_test_transaction(
                "t1",
                "2024-01-10",
                -5000,
                ReconciliationStatus::Cleared,
            )],
            input_mode: InputMode::TransactionForm,
            form_state: Some(TransactionFormState::new("acc1".to_string(), "YYYY-MM-DD")),
            ..Default::default()
        }))];

        let new_transaction =
            create_test_transaction("t2", "2024-01-15", -3000, ReconciliationStatus::Uncleared);

        reduce_data_event(
            &mut state,
            DataEvent::TransactionCreated {
                transaction: new_transaction,
            },
        );

        let Screen::Transactions(trans_state) = state.current_screen() else {
            panic!("Expected Transactions screen");
        };
        // Should have 2 transactions
        assert_eq!(trans_state.transactions.len(), 2);
        // Should be sorted with new one first (newer date)
        assert_eq!(
            trans_state.transactions[0].date,
            NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap()
        );
        // Form should be closed
        assert_eq!(trans_state.input_mode, InputMode::Normal);
        assert!(trans_state.form_state.is_none());
        // Selection should be reset to first item
        assert_eq!(trans_state.table_state.borrow().selected(), Some(0));
    }

    #[test]
    fn test_transaction_create_failed() {
        let mut state = AppState::new();
        state.history = vec![Screen::Transactions(Box::new(TransactionsState {
            input_mode: InputMode::TransactionForm,
            form_state: Some(TransactionFormState::new("acc1".to_string(), "YYYY-MM-DD")),
            ..Default::default()
        }))];

        reduce_data_event(
            &mut state,
            DataEvent::TransactionCreateFailed {
                error: "Validation error: amount required".to_string(),
            },
        );

        if let Screen::Transactions(trans_state) = state.current_screen() {
            // Form should still be open
            assert_eq!(trans_state.input_mode, InputMode::TransactionForm);
            // Error should be set
            let Some(ref form) = trans_state.form_state else {
                panic!("Expected form_state to be Some");
            };
            assert_eq!(
                form.validation_error,
                Some("Validation error: amount required".to_string())
            );
        } else {
            panic!("Expected Transactions screen");
        }
    }

    #[test]
    fn test_transaction_updated_full() {
        let mut state = AppState::new();
        state.history = vec![Screen::Transactions(Box::new(TransactionsState {
            transactions: vec![create_test_transaction(
                "t1",
                "2024-01-10",
                -5000,
                ReconciliationStatus::Cleared,
            )],
            input_mode: InputMode::TransactionForm,
            form_state: Some(TransactionFormState::new("acc1".to_string(), "YYYY-MM-DD")),
            ..Default::default()
        }))];

        let updated_transaction =
            create_test_transaction("t1", "2024-01-15", -7000, ReconciliationStatus::Uncleared);

        reduce_data_event(
            &mut state,
            DataEvent::TransactionUpdatedFull {
                transaction: updated_transaction,
            },
        );

        let Screen::Transactions(trans_state) = state.current_screen() else {
            panic!("Expected Transactions screen");
        };
        assert_eq!(trans_state.transactions.len(), 1);
        assert_eq!(trans_state.transactions[0].amount, Milliunits::new(-7000));
        assert_eq!(
            trans_state.transactions[0].date,
            NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap()
        );
        // Form should be closed
        assert_eq!(trans_state.input_mode, InputMode::Normal);
        assert!(trans_state.form_state.is_none());
    }

    #[test]
    fn test_transactions_delta_merge() {
        let mut state = AppState::new();
        state.history = vec![Screen::Transactions(Box::new(TransactionsState {
            transactions: vec![
                create_test_transaction("t1", "2024-01-15", -5000, ReconciliationStatus::Cleared),
                create_test_transaction("t2", "2024-01-10", -3000, ReconciliationStatus::Uncleared),
            ],
            ..Default::default()
        }))];

        // Delta: update t1, delete t2, add t3
        let updated_t1 =
            create_test_transaction("t1", "2024-01-15", -6000, ReconciliationStatus::Cleared);
        let mut deleted_t2 =
            create_test_transaction("t2", "2024-01-10", -3000, ReconciliationStatus::Uncleared);
        deleted_t2.deleted = true;
        let new_t3 =
            create_test_transaction("t3", "2024-01-20", -2000, ReconciliationStatus::Cleared);

        reduce_data_event(
            &mut state,
            DataEvent::TransactionsDeltaLoaded {
                delta: vec![updated_t1, deleted_t2, new_t3],
            },
        );

        let Screen::Transactions(trans_state) = state.current_screen() else {
            // Should have t3 (newest) and t1, t2 should be deleted
            panic!("Expected Transactions screen");
        };
        assert_eq!(trans_state.transactions.len(), 2);
        // Verify order: newest first (2024-01-20), then 2024-01-15
        assert_eq!(
            trans_state.transactions[0].date,
            NaiveDate::parse_from_str("2024-01-20", "%Y-%m-%d").unwrap()
        );
        assert_eq!(
            trans_state.transactions[1].date,
            NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d").unwrap()
        );
        assert_eq!(trans_state.transactions[1].amount, Milliunits::new(-6000)); // updated amount
    }

    // ============================================================================
    // Payees and Categories Tests
    // ============================================================================

    #[test]
    fn test_payees_loaded() {
        let mut state = AppState::new();
        state.history = vec![Screen::Transactions(Box::default())];

        let payees = vec![Payee {
            id: Uuid::new_v4(),
            name: "Grocery Store".to_string(),
            transfer_account_id: None,
            deleted: false,
        }];

        reduce_data_event(
            &mut state,
            DataEvent::PayeesLoaded {
                payees: payees.clone(),
            },
        );

        let Screen::Transactions(trans_state) = state.current_screen() else {
            panic!("Expected Transactions screen");
        };
        assert_eq!(trans_state.payees.len(), 1);
        assert_eq!(trans_state.payees[0].name, "Grocery Store");
    }

    #[test]
    fn test_categories_loaded() {
        let mut state = AppState::new();
        state.history = vec![Screen::Transactions(Box::default())];

        let categories = vec![Category {
            id: Uuid::new_v4(),
            category_group_id: Uuid::new_v4(),
            category_group_name: Some("Monthly".to_string()),
            name: "Groceries".to_string(),
            hidden: false,
            original_category_group_id: None,
            note: None,
            budgeted: 0.into(),
            activity: 0.into(),
            balance: 0.into(),
            goal_type: None,
            goal_creation_month: None,
            goal_target: None,
            goal_target_month: None,
            goal_percentage_complete: None,
            goal_months_to_budget: None,
            goal_under_funded: None,
            goal_overall_funded: None,
            goal_overall_left: None,
            goal_snoozed_at: None,
            deleted: false,
        }];

        reduce_data_event(
            &mut state,
            DataEvent::CategoriesLoaded {
                categories: categories.clone(),
            },
        );

        let Screen::Transactions(trans_state) = state.current_screen() else {
            panic!("Expected Transactions screen");
        };
        assert_eq!(trans_state.categories.len(), 1);
        assert_eq!(trans_state.categories[0].name, "Groceries");
    }

    // ============================================================================
    // Error Handling Tests
    // ============================================================================

    #[test]
    fn test_load_error_on_accounts_screen() {
        let mut state = AppState::new();
        state.history = vec![Screen::Accounts(AccountsState {
            accounts_loading: LoadingState::Loading(throbber_widgets_tui::ThrobberState::default()),
            ..Default::default()
        })];

        reduce_data_event(
            &mut state,
            DataEvent::LoadError {
                error: "Network error".to_string(),
            },
        );

        let Screen::Accounts(accounts_state) = state.current_screen() else {
            panic!("Expected Accounts screen");
        };
        match &accounts_state.accounts_loading {
            LoadingState::Error(msg) => assert_eq!(msg, "Network error"),
            _ => panic!("Expected Error loading state"),
        }
    }

    #[test]
    fn test_load_error_on_budgets_screen() {
        let mut state = AppState::new();
        state.history = vec![Screen::Budgets(BudgetsState {
            budgets_loading: LoadingState::Loading(throbber_widgets_tui::ThrobberState::default()),
            ..Default::default()
        })];

        reduce_data_event(
            &mut state,
            DataEvent::LoadError {
                error: "API timeout".to_string(),
            },
        );

        let Screen::Budgets(budgets_state) = state.current_screen() else {
            panic!("Expected Budgets screen");
        };
        match &budgets_state.budgets_loading {
            LoadingState::Error(msg) => assert_eq!(msg, "API timeout"),
            _ => panic!("Expected Error loading state"),
        }
    }
}
