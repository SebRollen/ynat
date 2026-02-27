use super::{BudgetId, LastKnowledgeOfServer, LastKnowledgeQuery};
use crate::{endpoints::Milliunits, macros::setter};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tower_api_client::{Method, Request, RequestData};
use uuid::Uuid;

// Common

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub category_group_id: Uuid,
    pub category_group_name: Option<String>,
    pub name: String,
    pub hidden: bool,
    pub original_category_group_id: Option<Uuid>,
    pub note: Option<String>,
    pub budgeted: Milliunits,
    pub activity: Milliunits,
    pub balance: Milliunits,
    pub goal_type: Option<String>,
    pub goal_creation_month: Option<String>,
    pub goal_target: Option<Milliunits>,
    pub goal_target_month: Option<String>,
    pub goal_percentage_complete: Option<i32>,
    pub goal_months_to_budget: Option<i32>,
    pub goal_under_funded: Option<Milliunits>,
    pub goal_overall_funded: Option<Milliunits>,
    pub goal_overall_left: Option<Milliunits>,
    pub goal_snoozed_at: Option<DateTime<Utc>>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryGroup {
    pub id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub deleted: bool,
    pub categories: Vec<Category>,
}

// Requests

#[derive(Default, Debug, Clone, Serialize)]
pub struct ListCategories {
    budget_id: BudgetId,
    #[serde(skip)]
    last_knowledge_query: Option<LastKnowledgeQuery>,
}

impl ListCategories {
    pub fn new() -> Self {
        Self::default()
    }

    setter!(budget_id: BudgetId);

    pub fn last_knowledge_of_server(mut self, value: LastKnowledgeOfServer) -> Self {
        self.last_knowledge_query = Some(LastKnowledgeQuery::from(&value));
        self
    }
}

impl Request for ListCategories {
    type Data = LastKnowledgeQuery;
    type Response = ListCategoriesResponse;

    fn endpoint(&self) -> Cow<'_, str> {
        format!("/budgets/{}/categories", self.budget_id).into()
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
pub struct UpdateMonthCategory {
    #[serde(skip)]
    budget_id: BudgetId,
    #[serde(skip)]
    month: String,
    #[serde(skip)]
    category_id: Uuid,
    category: SaveMonthCategory,
}

impl UpdateMonthCategory {
    pub fn new(category_id: Uuid, budgeted: Milliunits) -> Self {
        Self {
            budget_id: BudgetId::default(),
            month: String::new(),
            category_id,
            category: SaveMonthCategory { budgeted },
        }
    }

    setter!(budget_id: BudgetId);

    pub fn month(mut self, month: impl Into<String>) -> Self {
        self.month = month.into();
        self
    }
}

impl Request for UpdateMonthCategory {
    type Data = Self;
    type Response = SaveCategoryResponse;
    const METHOD: Method = Method::PATCH;

    fn endpoint(&self) -> Cow<'_, str> {
        format!(
            "/budgets/{}/months/{}/categories/{}",
            self.budget_id, self.month, self.category_id
        )
        .into()
    }

    fn data(&self) -> RequestData<&Self::Data> {
        RequestData::Json(self)
    }
}

// Responses

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCategoriesResponse {
    pub data: CategoriesData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriesData {
    pub category_groups: Vec<CategoryGroup>,
    pub server_knowledge: Option<LastKnowledgeOfServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMonthCategoryRequest {
    pub category: SaveMonthCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMonthCategory {
    pub budgeted: Milliunits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveCategoryResponse {
    pub data: SaveCategoryData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveCategoryData {
    pub category: Category,
}
