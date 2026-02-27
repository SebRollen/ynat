use serde::{Deserialize, Serialize};
use tower_api_client::{Error as ApiError, StatusCode};

#[derive(Debug)]
pub enum YnabApiError {
    Ynab(StatusCode, ErrorDetail),
    Internal(ApiError),
}

impl From<ApiError> for YnabApiError {
    fn from(value: ApiError) -> Self {
        match value {
            ApiError::ClientError(status, detail) | ApiError::ServerError(status, detail) => {
                let response: ErrorResponse = serde_json::from_str(&detail).unwrap();
                YnabApiError::Ynab(status, response.error)
            }
            e => YnabApiError::Internal(e),
        }
    }
}

impl std::fmt::Display for YnabApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YnabApiError::Internal(e) => write!(f, "Internal error: {}", e),
            YnabApiError::Ynab(status, detail) => {
                write!(f, "({}) {}: {}", status, detail.name, detail.detail)
            }
        }
    }
}

impl std::error::Error for YnabApiError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub id: String,
    pub name: String,
    pub detail: String,
}
