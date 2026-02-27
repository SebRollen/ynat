use super::accounts::AccountSummary;
use super::{BudgetId, CurrencyFormat, DateFormat};
use crate::macros::setter;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tower_api_client::{Request, RequestData};

// Common

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetSummary {
    pub id: BudgetId,
    pub name: String,
    pub last_modified_on: Option<DateTime<Utc>>,
    pub first_month: Option<NaiveDate>,
    pub last_month: Option<NaiveDate>,
    pub date_format: Option<DateFormat>,
    pub currency_format: Option<CurrencyFormat>,
    pub accounts: Option<Vec<AccountSummary>>,
}

// Requests

#[derive(Default, Debug, Clone, Serialize)]
pub struct ListBudgets {
    include_accounts: bool,
}

impl ListBudgets {
    pub fn new() -> Self {
        Self::default()
    }

    setter!(include_accounts: bool);
}

impl Request for ListBudgets {
    type Data = Self;
    type Response = BudgetsResponse;

    fn endpoint(&self) -> Cow<'_, str> {
        "/budgets".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Query(self)
    }
}

// Responses

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetsResponse {
    pub data: BudgetsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetsData {
    pub budgets: Vec<BudgetSummary>,
    pub default_budget: Option<BudgetSummary>,
}
