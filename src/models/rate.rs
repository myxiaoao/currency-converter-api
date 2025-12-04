use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Daily exchange rates with EUR as base currency (from ECB)
/// Uses Decimal for precise financial calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRate {
    pub date: String,
    pub base: String,
    pub rates: HashMap<String, Decimal>,
}

/// ECB XML envelope structure
#[derive(Debug, Deserialize)]
#[serde(rename = "Envelope")]
pub struct EcbEnvelope {
    #[serde(rename = "Cube")]
    pub cube: EcbOuterCube,
}

#[derive(Debug, Deserialize)]
pub struct EcbOuterCube {
    #[serde(rename = "Cube")]
    pub time_cube: EcbTimeCube,
}

#[derive(Debug, Deserialize)]
pub struct EcbTimeCube {
    #[serde(rename = "@time")]
    pub time: String,
    #[serde(rename = "Cube", default)]
    pub rates: Vec<EcbRate>,
}

#[derive(Debug, Deserialize)]
pub struct EcbRate {
    #[serde(rename = "@currency")]
    pub currency: String,
    #[serde(rename = "@rate")]
    pub rate: String,
}

impl DailyRate {
    pub fn from_ecb_data(time: String, rates: Vec<EcbRate>) -> Result<Self, String> {
        let mut rate_map = HashMap::new();

        // Add all rates from ECB
        for rate in rates {
            let rate_value = rate
                .rate
                .parse::<Decimal>()
                .map_err(|e| format!("Failed to parse rate for {}: {}", rate.currency, e))?;
            rate_map.insert(rate.currency.to_uppercase(), rate_value);
        }

        // Add EUR = 1.0 (ECB doesn't include it since it's the base)
        rate_map.insert("EUR".to_string(), Decimal::ONE);

        Ok(DailyRate {
            date: time,
            base: "EUR".to_string(),
            rates: rate_map,
        })
    }

    /// Validate that the date is in correct format
    pub fn validate_date(&self) -> Result<(), String> {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid date format: {}", e))?;
        Ok(())
    }
}
