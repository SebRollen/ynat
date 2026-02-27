use super::{BudgetId, categories::Category};
use crate::endpoints::{LastKnowledgeOfServer, Milliunits};
use crate::macros::setter;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Display;
use tower_api_client::Request;

// Common

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthDetail {
    pub month: String,
    pub note: Option<String>,
    pub income: Milliunits,
    pub budgeted: Milliunits,
    pub activity: Milliunits,
    pub to_be_budgeted: Milliunits,
    pub age_of_money: Option<i64>,
    pub deleted: bool,
    pub categories: Vec<Category>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum Month {
    #[default]
    #[serde(rename = "current")]
    Current,
    #[serde(untagged)]
    Month(String),
}

impl Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Current => f.write_str("current"),
            Self::Month(month) => f.write_str(month),
        }
    }
}

// Requests

#[derive(Default, Debug, Clone, Serialize)]
pub struct GetMonth {
    budget_id: BudgetId,
    month: Month,
}

impl GetMonth {
    pub fn new() -> Self {
        Self::default()
    }

    setter!(budget_id: BudgetId);
    setter!(month: Month);
}

impl Request for GetMonth {
    type Data = ();
    type Response = GetMonthResponse;

    fn endpoint(&self) -> Cow<'_, str> {
        format!("/budgets/{}/months/{}", self.budget_id, self.month).into()
    }
}

// Responses

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMonthResponse {
    pub data: MonthData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthData {
    pub month: MonthDetail,
    pub server_knowledge: Option<LastKnowledgeOfServer>,
}

//pub async fn update_month_category(
//    &self,
//    budget_id: &str,
//    month: &str,
//    category_id: &str,
//    request: SaveMonthCategoryRequest,
//) -> Result<SaveCategoryResponse, ApiError> {
//    let url = format!(
//        "{}/budgets/{}/months/{}/categories/{}",
//        self.base_url, budget_id, month, category_id
//    );
//
//    self.execute_request(Method::PATCH, &url, Some(&request))
//        .await
//}
//
