use crate::error::ApiError;
use crate::models::{ConvertQuery, ConvertResponse};
use crate::services::{convert_currency, RedisStore};
use axum::{extract::{Query, State}, Json};
use validator::Validate;

pub async fn convert_handler(
    State(store): State<RedisStore>,
    Query(params): Query<ConvertQuery>,
) -> Result<Json<ConvertResponse>, ApiError> {
    // Validate query parameters
    params
        .validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Get rates from Redis
    let rates = store
        .get_rates()
        .await?
        .ok_or(ApiError::NoRatesAvailable)?;

    // Perform conversion
    let (result, rate) = convert_currency(&rates, &params.from, &params.to, params.amount)?;

    Ok(Json(ConvertResponse {
        from: params.from.to_uppercase(),
        to: params.to.to_uppercase(),
        amount: params.amount,
        result,
        rate,
        date: rates.date,
    }))
}
