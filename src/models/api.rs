use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Response for GET /api/latest
#[derive(Debug, Serialize)]
pub struct LatestRatesResponse {
    pub date: String,
    pub base: String,
    pub rates: HashMap<String, f64>,
}

/// Query parameters for GET /api/latest?base=USD
#[derive(Debug, Deserialize, Validate)]
pub struct LatestRatesQuery {
    #[validate(length(equal = 3))]
    pub base: Option<String>,
}

/// Query parameters for GET /api/convert
#[derive(Debug, Deserialize, Validate)]
pub struct ConvertQuery {
    #[validate(length(equal = 3))]
    pub from: String,
    #[validate(length(equal = 3))]
    pub to: String,
    #[validate(range(min = 0.0))]
    pub amount: f64,
}

/// Response for GET /api/convert
#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub result: f64,
    pub rate: f64,
    pub date: String,
}

/// Response for GET /health
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub redis: String,
    pub last_update: Option<String>,
}
