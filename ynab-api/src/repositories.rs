use crate::endpoints::{
    BudgetId, Milliunits, TransactionId,
    accounts::ListAccounts,
    budgets::ListBudgets,
    categories::{ListCategories, UpdateMonthCategory},
    months::GetMonth,
    payees::ListPayees,
    transactions::{
        BulkUpdateTransactions, CreateTransaction, DeleteTransaction, ListTransactions,
        UpdateTransaction,
    },
};
use uuid::Uuid;

#[derive(Default)]
pub struct AccountRepository {
    budget_id: BudgetId,
}

impl AccountRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_budget(mut self, budget_id: BudgetId) -> Self {
        self.budget_id = budget_id;
        self
    }

    pub fn list(&self) -> ListAccounts {
        ListAccounts::new(self.budget_id.clone())
    }
}

pub struct BudgetRepository;

impl BudgetRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub fn list(&self) -> ListBudgets {
        ListBudgets::default()
    }
}

#[derive(Default)]
pub struct CategoryRepository {
    budget_id: BudgetId,
}

impl CategoryRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_budget(mut self, budget_id: BudgetId) -> Self {
        self.budget_id = budget_id;
        self
    }

    pub fn list(&self) -> ListCategories {
        ListCategories::new().budget_id(self.budget_id.clone())
    }

    pub fn update_month(
        &self,
        category_id: Uuid,
        month: impl Into<String>,
        budgeted: Milliunits,
    ) -> UpdateMonthCategory {
        UpdateMonthCategory::new(category_id, budgeted)
            .budget_id(self.budget_id.clone())
            .month(month)
    }
}

pub struct MonthRepository;

impl MonthRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self) -> GetMonth {
        GetMonth::default()
    }
}

pub struct PayeeRepository;

impl PayeeRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub fn list(&self) -> ListPayees {
        ListPayees::default()
    }
}

#[derive(Default)]
pub struct TransactionRepository {
    budget_id: BudgetId,
}

impl TransactionRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_budget(mut self, budget_id: BudgetId) -> Self {
        self.budget_id = budget_id;
        self
    }

    pub fn bulk(&self) -> BulkTransactionRepository {
        BulkTransactionRepository::new()
    }

    pub fn list(&self, account_id: Uuid) -> ListTransactions {
        ListTransactions::new(account_id).budget_id(self.budget_id.clone())
    }

    pub fn create(&self, account_id: Uuid, date: String, amount: i64) -> CreateTransaction {
        CreateTransaction::new(account_id, date, amount).budget_id(self.budget_id.clone())
    }

    pub fn update(&self, transaction_id: TransactionId) -> UpdateTransaction {
        UpdateTransaction::new(transaction_id).budget_id(self.budget_id.clone())
    }

    pub fn delete(&self, transaction_id: TransactionId) -> DeleteTransaction {
        DeleteTransaction::new(transaction_id).budget_id(self.budget_id.clone())
    }
}

pub struct BulkTransactionRepository;

impl BulkTransactionRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&self) -> BulkUpdateTransactions {
        BulkUpdateTransactions::new()
    }
}
