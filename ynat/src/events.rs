use ynab_api::endpoints::{
    accounts::Account,
    budgets::BudgetSummary,
    categories::Category,
    months::MonthDetail,
    payees::Payee,
    transactions::{ReconciliationStatus, Transaction},
};

/// Commands to execute (user actions → background tasks)
#[derive(Debug, Clone, PartialEq)]
pub enum AppCommand {
    SelectNext,
    SelectPrevious,
    NavigateToTop,
    NavigateToBottom,

    // Navigation
    NavigateBack,

    // Data loading
    LoadBudgets {
        force_refresh: bool,
        load_accounts: bool,
    },
    LoadAccounts {
        budget_id: String,
        budget: Box<Option<BudgetSummary>>,
        force_refresh: bool,
    },
    LoadTransactions {
        budget_id: String,
        account_id: String,
        force_refresh: bool,
    },
    LoadPlan {
        budget_id: String,
        force_refresh: bool,
    },
    LoadPlanMonth {
        budget_id: String,
        month: String,
    },
    NavigatePlanMonth {
        forward: bool,
    },

    // Transaction updates
    ToggleTransactionCleared {
        transaction_id: String,
        budget_id: String,
    },

    // Transaction creation
    EnterTransactionCreateMode,
    ExitTransactionCreateMode,
    NavigateFormField {
        forward: bool,
    },
    AppendFormFieldChar {
        c: char,
    },
    DeleteFormFieldChar,
    ClearFormField,
    SelectAutocompleteItem {
        up: bool,
    },
    ConfirmAutocompleteSelection,
    SubmitTransactionForm,

    // Split transaction mode
    EnterSplitMode,
    AddSubtransaction,
    DeleteSubtransaction,
    LoadPayees {
        budget_id: String,
    },
    LoadCategories {
        budget_id: String,
    },

    ApproveTransaction {
        budget_id: String,
        transaction_id: String,
    },

    // Transaction deletion
    InitiateTransactionDelete {
        transaction_id: String,
    },
    ConfirmTransactionDelete {
        transaction_id: String,
        budget_id: String,
    },
    CancelTransactionDelete,

    // Transaction editing
    InitiateTransactionEdit {
        transaction_id: String,
    },
    ConfirmReconciledEdit {
        transaction_id: String,
    },
    CancelReconciledEdit,
    EnterTransactionEditMode {
        transaction_id: String,
    },

    // Reconciliation
    InitiateReconcile {
        cleared_balance: i64,
    },
    ConfirmReconcile {
        budget_id: String,
        account_id: String,
    },
    CancelReconcile,

    // Filter mode
    EnterFilterMode,
    ExitFilterMode,
    AppendFilterChar(char),
    DeleteFilterChar,
    ClearFilter,

    // View toggles
    ToggleShowClosedAccounts,
    ToggleShowReconciledTransactions,
    TogglePlanFocusedView,
    ToggleHelp,

    // Log screen
    NavigateToLogs,
    ScrollLogsUp,
    ScrollLogsDown,
    ScrollLogsPageUp,
    ScrollLogsPageDown,
    ScrollLogsToTop,
    ScrollLogsToBottom,

    // Key sequence state
    SetPendingKey(char),
    ClearPendingKey,

    // Budget editing (Plan screen)
    InitiateBudgetEdit {
        category_id: String,
    },
    ExitBudgetEditMode,
    AppendBudgetChar(char),
    DeleteBudgetChar,
    SubmitBudgetEdit {
        budget_id: String,
        month: String,
    },

    // System
    Quit,
}

/// Events from background tasks (responses to commands)
#[derive(Debug, Clone)]
pub enum DataEvent {
    // Cache events (instant)
    BudgetsCacheLoaded {
        budgets: Vec<BudgetSummary>,
        default_budget: Option<BudgetSummary>,
    },
    AccountsCacheLoaded {
        accounts: Vec<Account>,
    },
    TransactionsCacheLoaded {
        transactions: Vec<Transaction>,
    },

    // API events (slower)
    BudgetsLoaded {
        budgets: Vec<BudgetSummary>,
        default_budget: Option<BudgetSummary>,
    },
    AccountsLoaded {
        accounts: Vec<Account>,
    },
    TransactionsLoaded {
        transactions: Vec<Transaction>,
    },

    // Delta updates (background refresh)
    AccountsDeltaLoaded {
        delta: Vec<Account>,
    },
    TransactionsDeltaLoaded {
        delta: Vec<Transaction>,
    },

    // Plan data
    PlanCacheLoaded {
        month: MonthDetail,
        categories: Vec<Category>,
    },
    PlanLoaded {
        month: MonthDetail,
        categories: Vec<Category>,
    },

    // Transaction approval
    TransactionApproved {
        transaction_id: String,
    },
    TransactionApproveFailed {
        transaction_id: String,
        error: String,
    },

    // Transaction updates
    TransactionUpdated {
        transaction_id: String,
    },
    TransactionUpdateFailed {
        transaction_id: String,
        original_status: ReconciliationStatus,
        original_approved: bool,
        error: String,
    },

    // Transaction creation
    PayeesLoaded {
        payees: Vec<Payee>,
    },
    CategoriesLoaded {
        categories: Vec<Category>,
    },
    TransactionCreated {
        transaction: Transaction,
    },
    TransactionCreateFailed {
        error: String,
    },

    // Transaction deletion
    TransactionDeleted {
        transaction_id: String,
    },
    TransactionDeleteFailed {
        transaction_id: String,
        error: String,
    },

    // Transaction editing (full update)
    TransactionUpdatedFull {
        transaction: Transaction,
    },
    TransactionUpdateFullFailed {
        transaction_id: String,
        error: String,
    },

    // Reconciliation
    TransactionsReconciled {
        transaction_ids: Vec<String>,
    },
    TransactionsReconcileFailed {
        error: String,
    },

    // Budget category updates
    CategoryBudgetUpdated {
        category: Category,
    },
    CategoryBudgetUpdateFailed {
        category_id: String,
        original_budgeted: i64,
        new_budgeted: i64,
        error: String,
    },

    // Errors
    LoadError {
        error: String,
    },
}
