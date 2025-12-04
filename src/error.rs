use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Currency code '{0}' not found in exchange rates")]
    CurrencyNotFound(String),

    #[error("No exchange rates available. Please try again later.")]
    NoRatesAvailable,

    #[error("Invalid parameter: {0}")]
    ValidationError(String),

    #[error("Failed to fetch ECB data: {0}")]
    EcbFetchError(String),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("XML parse error: {0}")]
    XmlParseError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::CurrencyNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::NoRatesAvailable => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            ApiError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::EcbFetchError(ref msg) => {
                tracing::error!("ECB fetch error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to fetch exchange rates".to_string(),
                )
            }
            ApiError::RedisError(ref err) => {
                tracing::error!("Redis error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database connection error".to_string(),
                )
            }
            ApiError::XmlParseError(ref msg) => {
                tracing::error!("XML parse error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to parse exchange rate data".to_string(),
                )
            }
            ApiError::InternalError(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

// Helper to convert anyhow errors
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::InternalError(err.to_string())
    }
}
