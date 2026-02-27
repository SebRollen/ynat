pub mod endpoints;
mod error;
mod macros;
pub mod repositories;

pub use crate::error::YnabApiError;
use repositories::*;
use tower_api_client::{Client as ApiClient, Request as ApiRequest};

const BASE_URL: &str = "https://api.ynab.com/v1";
//const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub struct Client {
    inner: ApiClient,
}

impl Client {
    pub fn new(access_token: &str) -> Self {
        Self {
            inner: ApiClient::new(BASE_URL).bearer_auth(access_token),
        }
    }

    pub async fn send<R>(&self, request: R) -> Result<R::Response, YnabApiError>
    where
        R: ApiRequest,
    {
        self.inner.send(request).await.map_err(From::from)
    }
}

pub struct Request;

impl Request {
    pub fn new() -> Self {
        Self {}
    }

    pub fn accounts() -> AccountRepository {
        AccountRepository::new()
    }

    pub fn budgets() -> BudgetRepository {
        BudgetRepository::new()
    }

    pub fn categories() -> CategoryRepository {
        CategoryRepository::new()
    }

    pub fn months() -> MonthRepository {
        MonthRepository::new()
    }

    pub fn payees() -> PayeeRepository {
        PayeeRepository::new()
    }

    pub fn transactions() -> TransactionRepository {
        TransactionRepository::new()
    }
}
