use crate::error::ApiError;
use crate::models::{ConvertQuery, ConvertResponse};
use crate::services::{RedisStore, convert_currency};
use axum::{
    Json,
    extract::{Query, State},
};
use validator::Validate;

pub async fn convert_handler(
    State(store): State<RedisStore>,
    Query(params): Query<ConvertQuery>,
) -> Result<Json<ConvertResponse>, ApiError> {
    // Validate query parameters
    params
        .validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Parse and validate amount
    let amount = params
        .parse_amount()
        .map_err(|e| ApiError::ValidationError(e))?;

    // Get rates from Redis
    let rates = store.get_rates().await?.ok_or(ApiError::NoRatesAvailable)?;

    // Perform conversion (optimized O(1) direct calculation)
    let (result, rate) = convert_currency(&rates, &params.from, &params.to, amount)?;

    Ok(Json(ConvertResponse {
        from: params.from.to_uppercase(),
        to: params.to.to_uppercase(),
        amount,
        result,
        rate,
        date: rates.date,
    }))
}
