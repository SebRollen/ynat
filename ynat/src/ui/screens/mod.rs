pub mod accounts_screen;
pub mod budgets_screen;
pub mod logs_screen;
pub mod plan_screen;
pub mod transactions_screen;

use crate::state::{AccountsState, BudgetsState, LogsState, PlanState, TransactionsState};

#[derive(Debug, Clone)]
pub enum Screen {
    Budgets(BudgetsState),
    Accounts(AccountsState),
    Transactions(Box<TransactionsState>),
    Plan(PlanState),
    Logs(LogsState),
}
