use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use validator::Validate;

/// Response for GET /api/latest
#[derive(Debug, Serialize)]
pub struct LatestRatesResponse {
    pub date: String,
    pub base: String,
    pub rates: HashMap<String, Decimal>,
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
    pub amount: String, // Accept as string to parse as Decimal for precision
}

impl ConvertQuery {
    /// Parse amount string to Decimal with validation
    pub fn parse_amount(&self) -> Result<Decimal, String> {
        let amount =
            Decimal::from_str(&self.amount).map_err(|e| format!("Invalid amount format: {}", e))?;

        if amount < Decimal::ZERO {
            return Err("Amount must be non-negative".to_string());
        }

        Ok(amount)
    }
}

/// Response for GET /api/convert
#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    pub from: String,
    pub to: String,
    pub amount: Decimal,
    pub result: Decimal,
    pub rate: Decimal,
    pub date: String,
}

/// Response for GET /health
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub redis: String,
    pub last_update: Option<String>,
}
