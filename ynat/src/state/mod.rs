pub mod autocomplete;
pub mod reducer;
pub mod validators;

use crate::ui::screens::Screen;
use crate::ui::utils as ui_utils;
use itertools::Itertools;
use ratatui::widgets::TableState;
use std::cell::RefCell;
use throbber_widgets_tui::ThrobberState;
use ynab_api::endpoints::{
    accounts::Account,
    budgets::BudgetSummary,
    categories::Category,
    months::MonthDetail,
    payees::Payee,
    transactions::{FlagColor, ReconciliationStatus, SubTransaction, Transaction},
};

/// Represents loading state separate from data state
#[derive(Default, Debug, Clone, PartialEq)]
pub enum LoadingState {
    #[default]
    NotStarted,
    Loading(ThrobberState),
    Loaded,
    Error(String),
}

/// Represents input mode for screens that support editing
#[derive(Default, Debug, Clone, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Filter,
    TransactionForm,
    DeleteConfirmation,
    ReconciledEditConfirmation,
    ReconcileConfirmation,
    BudgetEdit,
}

/// Focused view filter for Plan screen categories
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum PlanFocusedView {
    #[default]
    All,
    Snoozed,
    Underfunded,
    Overfunded,
    MoneyAvailable,
}

impl PlanFocusedView {
    /// Cycle to the next focused view
    pub fn next(&self) -> Self {
        match self {
            Self::All => Self::Underfunded,
            Self::Underfunded => Self::Overfunded,
            Self::Overfunded => Self::Snoozed,
            Self::Snoozed => Self::MoneyAvailable,
            Self::MoneyAvailable => Self::All,
        }
    }

    /// Display name for the view
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Snoozed => "Snoozed",
            Self::Underfunded => "Underfunded",
            Self::Overfunded => "Overfunded",
            Self::MoneyAvailable => "Money Available",
        }
    }
}

/// Form field for transaction creation
#[derive(Debug, Clone, PartialEq)]
pub enum FormField {
    Date,
    Amount,
    Payee,
    Category,
    Memo,
    FlagColor,
    Cleared,
}

/// Form field for subtransaction editing
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SubTransactionField {
    #[default]
    Amount,
    Category,
    Memo,
}

/// State for a single subtransaction in split mode
#[derive(Debug, Clone)]
pub struct SubTransactionFormState {
    pub amount: String,
    pub category: String,
    pub memo: String,
    pub filtered_categories: Vec<Category>,
    pub category_selection_index: usize,
}

impl SubTransactionFormState {
    pub fn new() -> Self {
        Self {
            amount: String::new(),
            category: String::new(),
            memo: String::new(),
            filtered_categories: Vec::new(),
            category_selection_index: 0,
        }
    }

    pub fn from_subtransaction(sub: &SubTransaction) -> Self {
        Self {
            amount: format!("{:.2}", sub.amount.as_f64() / 1000.0),
            category: sub.category_name.clone().unwrap_or_default(),
            memo: sub.memo.clone().unwrap_or_default(),
            filtered_categories: Vec::new(),
            category_selection_index: 0,
        }
    }
}

impl Default for SubTransactionFormState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for transaction creation form
#[derive(Debug, Clone)]
pub struct TransactionFormState {
    pub account_id: String, // Account for this transaction
    pub current_field: Option<FormField>,
    pub date: String,     // YYYY-MM-DD format
    pub amount: String,   // User input as string
    pub payee: String,    // Text input for autocomplete
    pub category: String, // Text input for autocomplete
    pub memo: String,
    pub flag_color: Option<FlagColor>,
    pub cleared: ReconciliationStatus,

    // Autocomplete state
    pub filtered_payees: Vec<Payee>,
    pub filtered_categories: Vec<Category>,
    pub payee_selection_index: usize,
    pub category_selection_index: usize,

    // Edit mode tracking
    pub editing_transaction_id: Option<String>,

    // Validation errors
    pub validation_error: Option<String>,

    // Split mode state
    pub is_split_mode: bool,
    pub subtransactions: Vec<SubTransactionFormState>,
    pub active_subtransaction_index: Option<usize>,
    pub subtransaction_field: SubTransactionField,
}

impl TransactionFormState {
    pub fn new(account_id: String, date_format: &str) -> Self {
        // Default date to today in the user's preferred format
        let today_iso = chrono::Local::now().format("%Y-%m-%d").to_string();
        let today = ui_utils::fmt_date_with_format(&today_iso, date_format);

        Self {
            account_id,
            current_field: Some(FormField::Date),
            date: today,
            amount: String::new(),
            payee: String::new(),
            category: String::new(),
            memo: String::new(),
            flag_color: None,
            cleared: ReconciliationStatus::Uncleared,
            filtered_payees: Vec::new(),
            filtered_categories: Vec::new(),
            payee_selection_index: 0,
            category_selection_index: 0,
            editing_transaction_id: None,
            validation_error: None,
            is_split_mode: false,
            subtransactions: Vec::new(),
            active_subtransaction_index: None,
            subtransaction_field: SubTransactionField::default(),
        }
    }

    pub fn from_transaction(transaction: &Transaction, date_format: &str) -> Self {
        // Convert the transaction's ISO date to user's preferred format
        let date_iso = transaction.date.format("%Y-%m-%d").to_string();
        let date = ui_utils::fmt_date_with_format(&date_iso, date_format);

        // Check if transaction has subtransactions (split mode)
        let is_split = !transaction.subtransactions.is_empty();
        let subtransactions: Vec<SubTransactionFormState> = transaction
            .subtransactions
            .iter()
            .map(SubTransactionFormState::from_subtransaction)
            .collect();

        Self {
            account_id: transaction.account_id.to_string(),
            current_field: Some(FormField::Date),
            date,
            amount: format!("{:.2}", transaction.amount.as_f64() / 1000.0),
            payee: transaction.payee_name.clone().unwrap_or_default(),
            category: if is_split {
                String::new() // Category is shown as "Split (N)" in UI
            } else {
                transaction.category_name.clone().unwrap_or_default()
            },
            memo: transaction.memo.clone().unwrap_or_default(),
            flag_color: transaction.flag_color,
            cleared: transaction.cleared,
            filtered_payees: Vec::new(),
            filtered_categories: Vec::new(),
            payee_selection_index: 0,
            category_selection_index: 0,
            editing_transaction_id: Some(transaction.id.to_string()),
            validation_error: None,
            is_split_mode: is_split,
            subtransactions,
            active_subtransaction_index: None,
            subtransaction_field: SubTransactionField::default(),
        }
    }

    pub fn is_edit_mode(&self) -> bool {
        self.editing_transaction_id.is_some()
    }

    pub fn is_last_field_focused(&self) -> bool {
        self.current_field == Some(FormField::Cleared)
    }

    pub fn is_autocomplete_value_focused(&self) -> bool {
        match self.current_field {
            Some(FormField::Payee) if !self.filtered_payees.is_empty() => true,
            Some(FormField::Category) if !self.filtered_categories.is_empty() => true,
            _ => false,
        }
    }
}

/// State for budget editing form on plan screen
#[derive(Debug, Clone)]
pub struct BudgetFormState {
    pub category_id: String,
    pub category_name: String,
    pub budgeted_input: String, // User input as string (supports math expressions)
    pub original_budgeted: i64, // For cancel/rollback
    pub validation_error: Option<String>,
}

impl BudgetFormState {
    pub fn new(category_id: String, category_name: String, current_budgeted: i64) -> Self {
        Self {
            category_id,
            category_name,
            budgeted_input: format!("{:.2}", current_budgeted as f64 / 1000.0),
            original_budgeted: current_budgeted,
            validation_error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub history: Vec<Screen>,

    // Navigation state
    pub current_budget_id: Option<String>,
    pub current_budget: Option<BudgetSummary>,
    pub current_account_id: Option<String>,

    // UI state
    pub help_visible: bool,
    pub pending_key: Option<char>,

    // System
    pub should_quit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            history: vec![Screen::Accounts(AccountsState::default())],

            current_budget_id: None,
            current_budget: None,
            current_account_id: None,

            help_visible: false,
            pending_key: None,

            should_quit: false,
        }
    }

    /// Get the current screen (last in navigation stack)
    pub fn current_screen(&self) -> &Screen {
        self.history
            .last()
            .expect("Navigation stack should never be empty")
    }

    /// Get mutable reference to current screen
    pub fn current_screen_mut(&mut self) -> &mut Screen {
        self.history
            .last_mut()
            .expect("Navigation stack should never be empty")
    }

    /// Navigate to a new screen (push to stack)
    pub fn navigate_to(&mut self, screen: Screen) {
        tracing::debug!(
            "Navigating to new screen, stack depth: {} -> {}",
            self.history.len(),
            self.history.len() + 1
        );
        self.history.push(screen);
    }

    /// Navigate back (pop from stack)
    /// Returns true if navigation succeeded, false if already at root
    pub fn navigate_back(&mut self) -> bool {
        if self.history.len() > 1 {
            tracing::debug!(
                "Navigating back, stack depth: {} -> {}",
                self.history.len(),
                self.history.len() - 1
            );
            self.history.pop();
            true
        } else {
            tracing::debug!("Cannot navigate back, already at root screen");
            false
        }
    }

    pub fn loading_state(&mut self) -> Option<&mut ThrobberState> {
        match self.current_screen_mut() {
            Screen::Plan(state) => {
                if let LoadingState::Loading(ref mut throbber_state) = state.plan_loading {
                    return Some(throbber_state);
                }
            }
            Screen::Accounts(state) => {
                if let LoadingState::Loading(ref mut throbber_state) = state.accounts_loading {
                    return Some(throbber_state);
                }
            }
            Screen::Transactions(state) => {
                if let LoadingState::Loading(ref mut throbber_state) = state.transactions_loading {
                    return Some(throbber_state);
                }
            }
            Screen::Budgets(state) => {
                if let LoadingState::Loading(ref mut throbber_state) = state.budgets_loading {
                    return Some(throbber_state);
                }
            }
            Screen::Logs(_) => {
                // Logs screen has no loading state
            }
        }
        None
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug, Clone)]
pub struct BudgetsState {
    pub budgets: Vec<BudgetSummary>,
    pub budgets_loading: LoadingState,
    pub selected_budget_index: usize,
}

#[derive(Default, Debug, Clone)]
pub struct AccountsState {
    pub accounts: Vec<Account>,
    pub accounts_loading: LoadingState,
    pub table_state: RefCell<TableState>,
    pub input_mode: InputMode,
    pub filter_query: String,
    pub show_closed_accounts: bool,
}

#[derive(Debug, Clone)]
pub struct TransactionsState {
    pub accounts: Vec<Account>,
    pub transactions: Vec<Transaction>,
    pub transactions_loading: LoadingState,
    pub table_state: RefCell<TableState>,
    pub input_mode: InputMode,
    pub filter_query: String,
    pub show_reconciled_transactions: bool,

    // Transaction creation form
    pub form_state: Option<TransactionFormState>,
    pub payees: Vec<Payee>,
    pub categories: Vec<Category>,

    // Transaction deletion confirmation
    pub delete_confirmation_transaction_id: Option<String>,

    // Reconciled edit confirmation
    pub reconciled_edit_transaction_id: Option<String>,

    // Reconciliation confirmation
    pub reconcile_cleared_balance: Option<i64>,
}

impl Default for TransactionsState {
    fn default() -> Self {
        Self {
            accounts: Vec::default(),
            transactions: Vec::default(),
            transactions_loading: LoadingState::default(),
            table_state: RefCell::default(),
            input_mode: InputMode::default(),
            filter_query: String::default(),
            show_reconciled_transactions: true,
            form_state: Option::default(),
            payees: Vec::default(),
            categories: Vec::default(),
            delete_confirmation_transaction_id: Option::default(),
            reconciled_edit_transaction_id: Option::default(),
            reconcile_cleared_balance: Option::default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct PlanState {
    pub month: Option<MonthDetail>,
    pub categories: Vec<Category>,
    pub plan_loading: LoadingState,
    pub table_state: RefCell<TableState>,
    pub input_mode: InputMode,
    pub budget_form: Option<BudgetFormState>,
    pub focused_view: PlanFocusedView,
}

impl PlanState {
    /// Returns filtered categories based on the current focused view.
    /// Always filters out hidden and deleted categories.
    pub fn filtered_categories(&self) -> Vec<&Category> {
        self.categories
            .iter()
            .filter(|c| !c.hidden && !c.deleted)
            .filter(|c| match self.focused_view {
                PlanFocusedView::All => true,
                PlanFocusedView::Snoozed => c.goal_snoozed_at.is_some(),
                PlanFocusedView::Underfunded => {
                    if c.goal_snoozed_at.is_some() {
                        return false;
                    }
                    c.goal_under_funded
                        .map(|u| u.is_positive())
                        .unwrap_or(false)
                }
                PlanFocusedView::Overfunded => c
                    .goal_percentage_complete
                    .map(|pct| pct > 100)
                    .unwrap_or(false),
                PlanFocusedView::MoneyAvailable => c.balance.is_positive(),
            })
            .collect()
    }
}

#[derive(Default, Debug, Clone)]
pub struct LogsState {
    pub scroll_offset: usize,
    pub total_entries: usize,
}

impl AccountsState {
    /// Returns filtered accounts based on the current filter query.
    /// Optionally filters out deleted and closed accounts based on show_closed_accounts flag.
    pub fn filtered_accounts(&self) -> Vec<&Account> {
        let accounts: Vec<_> = self
            .accounts
            .iter()
            .filter(|a| self.show_closed_accounts || !a.closed)
            .collect();

        if self.filter_query.is_empty() {
            return accounts;
        }

        // Apply user's filter query
        let query_lower = self.filter_query.to_lowercase();
        accounts
            .into_iter()
            .filter(|a| {
                let name_match = a.name.to_lowercase().contains(&query_lower);
                let balance_str = format!("{:.2}", a.balance.as_f64() / 1000.0);
                let balance_match = balance_str.contains(&query_lower);
                name_match || balance_match
            })
            .collect()
    }
}

impl TransactionsState {
    /// Returns filtered transactions based on the current filter query.
    pub fn filtered_transactions(&self) -> Vec<&Transaction> {
        let transactions: Vec<_> = self
            .transactions
            .iter()
            .filter(|t| self.show_reconciled_transactions || !t.is_reconciled())
            .sorted()
            .collect();

        if self.filter_query.is_empty() {
            return transactions;
        }

        let query_lower = self.filter_query.to_lowercase();

        fn optional_match(opt: Option<&str>, search: &str) -> bool {
            let Some(req) = opt else {
                return false;
            };

            req.to_lowercase().contains(search)
        }

        transactions
            .into_iter()
            .filter(|t| {
                let payee_match = optional_match(t.payee_name.as_deref(), &query_lower);
                let category_match = optional_match(t.category_name.as_deref(), &query_lower);
                let memo_match = optional_match(t.memo.as_deref(), &query_lower);
                // TODO: this should match the budget format
                let amount_str = format!("{:.2}", t.amount.as_f64() / 1000.0);
                let amount_match = amount_str.contains(&query_lower);
                payee_match || category_match || memo_match || amount_match
            })
            .collect()
    }
}

pub trait Scrollable {
    fn num_items(&self) -> usize;
    fn table_state(&self) -> &RefCell<TableState>;

    fn select_prev(&mut self) {
        let mut table_state = self.table_state().borrow_mut();
        if self.num_items() > 0 {
            if table_state.selected().unwrap_or(0) == 0 {
                table_state.select_last();
            } else {
                table_state.scroll_up_by(1)
            }
        }
    }

    fn select_next(&mut self) {
        let num_items = self.num_items();
        let mut table_state = self.table_state().borrow_mut();
        if num_items > 0 {
            if table_state.selected().unwrap_or(num_items - 1) == num_items - 1 {
                table_state.select_first();
            } else {
                table_state.scroll_down_by(1)
            }
        }
    }
}

impl Scrollable for AccountsState {
    fn num_items(&self) -> usize {
        self.filtered_accounts().len()
    }

    fn table_state(&self) -> &RefCell<TableState> {
        &self.table_state
    }
}

impl Scrollable for TransactionsState {
    fn num_items(&self) -> usize {
        self.filtered_transactions().len()
    }

    fn table_state(&self) -> &RefCell<TableState> {
        &self.table_state
    }
}

impl Scrollable for PlanState {
    fn num_items(&self) -> usize {
        self.filtered_categories().len()
    }

    fn table_state(&self) -> &RefCell<TableState> {
        &self.table_state
    }
}
