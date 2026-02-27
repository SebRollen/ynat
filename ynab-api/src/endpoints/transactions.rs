use super::{BudgetId, LastKnowledgeOfServer, LastKnowledgeQuery, Milliunits, TransactionId};
use crate::macros::setter;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tower_api_client::{EmptyResponse, Method, Request, RequestData};
use uuid::Uuid;

// Common

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub id: TransactionId,
    pub date: NaiveDate,
    pub amount: Milliunits,
    pub memo: Option<String>,
    pub cleared: ReconciliationStatus,
    pub approved: bool,
    pub flag_color: Option<FlagColor>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<TransactionId>,
    pub matched_transaction_id: Option<TransactionId>,
    pub import_id: Option<String>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<SubTransaction>,
}

impl Transaction {
    pub fn is_reconciled(&self) -> bool {
        self.cleared == ReconciliationStatus::Reconciled
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .date
            .cmp(&self.date)
            .then(self.amount.cmp(&other.amount))
    }
}

// Requests

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTransactions {
    budget_id: BudgetId,
    account_id: Uuid,
    #[serde(skip)]
    last_knowledge_query: Option<LastKnowledgeQuery>,
}

impl ListTransactions {
    pub fn new(account_id: Uuid) -> Self {
        Self {
            account_id,
            budget_id: BudgetId::default(),
            last_knowledge_query: None,
        }
    }

    setter!(budget_id: BudgetId);

    pub fn last_knowledge_of_server(mut self, value: LastKnowledgeOfServer) -> Self {
        self.last_knowledge_query = Some(LastKnowledgeQuery::from(&value));
        self
    }
}

impl Request for ListTransactions {
    type Data = LastKnowledgeQuery;
    type Response = TransactionsResponse;

    fn endpoint(&self) -> Cow<'_, str> {
        format!(
            "/budgets/{}/accounts/{}/transactions",
            self.budget_id, self.account_id
        )
        .into()
    }

    fn data(&self) -> RequestData<&Self::Data> {
        if let Some(ref query) = self.last_knowledge_query {
            RequestData::Query(query)
        } else {
            RequestData::Empty
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTransaction {
    #[serde(skip)]
    budget_id: BudgetId,
    transaction: NewTransaction,
}

impl CreateTransaction {
    pub fn new<T>(account_id: Uuid, date: String, amount: T) -> Self
    where
        T: Into<Milliunits>,
    {
        let transaction = NewTransaction::new(account_id, date, amount);
        Self {
            budget_id: BudgetId::default(),
            transaction,
        }
    }

    setter!(budget_id: BudgetId);
    setter!(opt transaction.payee_id: Uuid);
    setter!(opt transaction.payee_name: String);
    setter!(opt transaction.category_id: Uuid);
    setter!(opt transaction.memo: String);
    setter!(opt transaction.cleared: ReconciliationStatus);
    setter!(opt transaction.approved: bool);
    setter!(opt transaction.flag_color: FlagColor);
    setter!(opt transaction.subtransactions: Vec<NewSubTransaction>);
}

impl Request for CreateTransaction {
    type Data = Self;
    type Response = CreateTransactionResponse;
    const METHOD: Method = Method::POST;

    fn endpoint(&self) -> Cow<'_, str> {
        format!("/budgets/{}/transactions", self.budget_id).into()
    }

    fn data(&self) -> RequestData<&Self::Data> {
        RequestData::Json(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransaction {
    pub account_id: Uuid,
    pub date: String,
    pub amount: Milliunits,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared: Option<ReconciliationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_color: Option<FlagColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtransactions: Option<Vec<NewSubTransaction>>,
}

impl NewTransaction {
    pub fn new<T>(account_id: Uuid, date: String, amount: T) -> Self
    where
        T: Into<Milliunits>,
    {
        Self {
            account_id,
            date,
            amount: amount.into(),
            payee_id: None,
            payee_name: None,
            category_id: None,
            memo: None,
            cleared: None,
            approved: None,
            flag_color: None,
            subtransactions: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSubTransaction {
    pub amount: Milliunits,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsResponse {
    pub data: TransactionsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsData {
    pub transactions: Vec<Transaction>,
    pub server_knowledge: Option<LastKnowledgeOfServer>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationStatus {
    Cleared,
    Uncleared,
    Reconciled,
}

impl std::fmt::Display for ReconciliationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cleared => write!(f, "Cleared"),
            Self::Uncleared => write!(f, "Uncleared"),
            Self::Reconciled => write!(f, "Reconciled"),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlagColor {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubTransaction {
    /// SubTransaction IDs are not UUIDs - they have a format like `{transaction_id}_{index}`
    pub id: String,
    pub transaction_id: TransactionId,
    pub amount: Milliunits,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

impl PartialOrd for SubTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SubTransaction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionResponse {
    pub data: CreateTransactionData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionData {
    pub transaction: Transaction,
    pub server_knowledge: Option<LastKnowledgeOfServer>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateTransaction {
    #[serde(skip)]
    budget_id: BudgetId,
    #[serde(skip)]
    transaction_id: TransactionId,
    transaction: TransactionUpdate,
}

impl UpdateTransaction {
    pub fn new(transaction_id: TransactionId) -> Self {
        Self {
            budget_id: BudgetId::default(),
            transaction_id,
            transaction: TransactionUpdate::default(),
        }
    }

    setter!(budget_id: BudgetId);
    setter!(opt transaction.account_id: Uuid);
    setter!(opt transaction.date: NaiveDate);
    setter!(opt transaction.amount: Milliunits);
    setter!(opt transaction.payee_id: Uuid);
    setter!(opt transaction.payee_name: String);
    setter!(opt transaction.category_id: Uuid);
    setter!(opt transaction.memo: String);
    setter!(opt transaction.flag_color: FlagColor);
    setter!(opt transaction.cleared: ReconciliationStatus);
    setter!(opt transaction.approved: bool);
    setter!(opt transaction.subtransactions: Vec<NewSubTransaction>);
}

impl Request for UpdateTransaction {
    type Data = Self;
    type Response = UpdateTransactionResponse;
    const METHOD: Method = Method::PUT;

    fn endpoint(&self) -> Cow<'_, str> {
        format!(
            "/budgets/{}/transactions/{}",
            self.budget_id, self.transaction_id
        )
        .into()
    }

    fn data(&self) -> RequestData<&Self::Data> {
        RequestData::Json(self)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TransactionUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Milliunits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payee_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub flag_color: Option<FlagColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared: Option<ReconciliationStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtransactions: Option<Vec<NewSubTransaction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransactionResponse {
    pub data: UpdateTransactionData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransactionData {
    pub transaction: Transaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTransaction {
    pub budget_id: BudgetId,
    pub transaction_id: TransactionId,
}

impl DeleteTransaction {
    pub fn new(transaction_id: TransactionId) -> Self {
        Self {
            budget_id: BudgetId::default(),
            transaction_id,
        }
    }

    setter!(budget_id: BudgetId);
}

impl Request for DeleteTransaction {
    type Data = ();
    type Response = EmptyResponse;
    const METHOD: Method = Method::DELETE;

    fn endpoint(&self) -> Cow<'_, str> {
        format!(
            "/budgets/{}/transactions/{}",
            self.budget_id, self.transaction_id
        )
        .into()
    }
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct BulkUpdateTransactions {
    #[serde(skip)]
    budget_id: BudgetId,
    transactions: Vec<BulkTransactionUpdate>,
}

impl BulkUpdateTransactions {
    pub fn new() -> Self {
        Self::default()
    }

    setter!(budget_id: BudgetId);

    pub fn transactions(mut self, transactions: Vec<BulkTransactionUpdate>) -> Self {
        self.transactions = transactions;
        self
    }
}

impl Request for BulkUpdateTransactions {
    type Data = Self;
    type Response = BulkUpdateTransactionsResponse;
    const METHOD: Method = Method::PATCH;

    fn endpoint(&self) -> Cow<'_, str> {
        format!("/budgets/{}/transactions", self.budget_id).into()
    }

    fn data(&self) -> RequestData<&Self::Data> {
        RequestData::Json(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTransactionUpdate {
    pub id: TransactionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleared: Option<ReconciliationStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkUpdateTransactionsResponse {
    pub data: BulkUpdateTransactionsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkUpdateTransactionsData {
    pub transactions: Vec<Transaction>,
}
