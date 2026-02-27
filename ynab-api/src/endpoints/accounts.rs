use super::{BudgetId, LastKnowledgeOfServer, LastKnowledgeQuery, Milliunits};
use crate::macros::setter;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tower_api_client::{Request, RequestData};
use uuid::Uuid;

// Common

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    /// Whether this account is on budget or not
    pub on_budget: bool,
    /// Whether this account is closed or not
    pub closed: bool,
    pub note: Option<String>,
    /// The current balance of the account in milliunits format
    pub balance: Milliunits,
    /// The current cleared balance of the account in milliunits format
    pub cleared_balance: Milliunits,
    /// The current uncleared balance of the account in milliunits format
    pub uncleared_balance: Milliunits,
    /// The payee id which should be used when transferring to this account
    pub transfer_payee_id: Option<Uuid>,
    /// Whether or not the account is linked to a financial institution for automatic transaction import.
    pub direct_import_linked: bool,
    /// If an account linked to a financial institution (direct_import_linked=true) and the linked connection is not in a healthy state, this will be true.
    pub direct_import_in_error: bool,
    /// Whether or not the account has been deleted. Deleted accounts will only be included in delta requests.
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub balance: Milliunits,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountType {
    Checking,
    Savings,
    Cash,
    CreditCard,
    LineOfCredit,
    OtherAsset,
    OtherLiability,
    Mortgage,
    AutoLoan,
    StudentLoan,
    PersonalLoan,
    MedicalDebt,
    OtherDebt,
}

// Requests

#[derive(Debug, Clone, Serialize)]
pub struct ListAccounts {
    budget_id: BudgetId,
    last_knowledge_query: Option<LastKnowledgeQuery>,
}

impl ListAccounts {
    pub fn new(budget_id: BudgetId) -> Self {
        Self {
            budget_id,
            last_knowledge_query: None,
        }
    }

    setter!(budget_id: BudgetId);

    pub fn last_knowledge_of_server(mut self, value: LastKnowledgeOfServer) -> Self {
        self.last_knowledge_query = Some(LastKnowledgeQuery::from(&value));
        self
    }
}

impl Request for ListAccounts {
    type Data = LastKnowledgeQuery;
    type Response = AccountsResponse;

    fn endpoint(&self) -> Cow<'_, str> {
        format!("/budgets/{}/accounts", self.budget_id).into()
    }

    fn data(&self) -> RequestData<&Self::Data> {
        if let Some(ref query) = self.last_knowledge_query {
            RequestData::Query(query)
        } else {
            RequestData::Empty
        }
    }
}

// Responses

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsResponse {
    pub data: AccountsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsData {
    pub accounts: Vec<Account>,
    pub server_knowledge: Option<LastKnowledgeOfServer>,
}
