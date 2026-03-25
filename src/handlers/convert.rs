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
    params
        .validate()
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    let amount = params.parse_amount()?;
    let from = params.from.to_uppercase();
    let to = params.to.to_uppercase();

    let rates = store.get_rates().await?.ok_or(ApiError::NoRatesAvailable)?;
    let (result, rate) = convert_currency(&rates, &from, &to, amount)?;

    Ok(Json(ConvertResponse {
        from,
        to,
        amount,
        result,
        rate,
        date: rates.date,
    }))
}
