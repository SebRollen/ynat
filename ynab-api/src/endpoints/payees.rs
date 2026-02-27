use super::{BudgetId, LastKnowledgeOfServer, LastKnowledgeQuery};
use crate::macros::setter;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tower_api_client::{Request, RequestData};
use uuid::Uuid;

// Common

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payee {
    pub id: Uuid,
    pub name: String,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

// Requests

#[derive(Default, Debug, Clone, Serialize)]
pub struct ListPayees {
    budget_id: BudgetId,
    #[serde(skip)]
    last_knowledge_query: Option<LastKnowledgeQuery>,
}

impl ListPayees {
    pub fn new() -> Self {
        Self::default()
    }

    setter!(budget_id: BudgetId);

    pub fn last_knowledge_of_server(mut self, value: LastKnowledgeOfServer) -> Self {
        self.last_knowledge_query = Some(LastKnowledgeQuery::from(&value));
        self
    }
}

impl Request for ListPayees {
    type Data = LastKnowledgeQuery;
    type Response = PayeesResponse;

    fn endpoint(&self) -> Cow<'_, str> {
        format!("/budgets/{}/payees", self.budget_id).into()
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
pub struct PayeesResponse {
    pub data: PayeesData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayeesData {
    pub payees: Vec<Payee>,
    pub server_knowledge: Option<LastKnowledgeOfServer>,
}
