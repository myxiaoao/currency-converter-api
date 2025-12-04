use crate::error::ApiError;
use crate::models::HealthResponse;
use crate::services::RedisStore;
use axum::{extract::State, Json};

pub async fn health_handler(
    State(store): State<RedisStore>,
) -> Result<Json<HealthResponse>, ApiError> {
    // Check Redis health
    let redis_status = match store.health_check().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Get last update date
    let last_update = store.get_last_update_date().await.ok().flatten();

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        redis: redis_status.to_string(),
        last_update,
    }))
}
