use crate::error::ApiError;
use crate::models::{LatestRatesQuery, LatestRatesResponse};
use crate::services::{rebase_rates, RedisStore};
use axum::{extract::{Query, State}, Json};
use validator::Validate;

pub async fn latest_rates_handler(
    State(store): State<RedisStore>,
    Query(params): Query<LatestRatesQuery>,
) -> Result<Json<LatestRatesResponse>, ApiError> {
    // Validate query parameters
    params
        .validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Get rates from Redis
    let rates = store
        .get_rates()
        .await?
        .ok_or(ApiError::NoRatesAvailable)?;

    // If base currency is specified, rebase the rates
    let result = if let Some(base) = params.base {
        let rebased = rebase_rates(&rates, &base)?;
        LatestRatesResponse {
            date: rebased.date,
            base: rebased.base,
            rates: rebased.rates,
        }
    } else {
        // Return rates with default EUR base
        LatestRatesResponse {
            date: rates.date,
            base: rates.base,
            rates: rates.rates,
        }
    };

    Ok(Json(result))
}
