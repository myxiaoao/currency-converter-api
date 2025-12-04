use crate::handlers::{convert_handler, health_handler, latest_rates_handler};
use crate::services::RedisStore;
use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::json;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

async fn root_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "service": "Currency Converter API",
            "version": "0.2.0",
            "endpoints": {
                "health": "GET /health",
                "latest_rates": "GET /api/latest?base=<CURRENCY>",
                "convert": "GET /api/convert?from=<FROM>&to=<TO>&amount=<AMOUNT>"
            }
        })),
    )
}

pub fn create_router(store: RedisStore) -> Router {
    // CORS configuration - adjust origins for production
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Root endpoint
        .route("/", get(root_handler))
        // Health check endpoint
        .route("/health", get(health_handler))
        // API endpoints
        .route("/api/latest", get(latest_rates_handler))
        .route("/api/convert", get(convert_handler))
        // Add shared state
        .with_state(store)
        // Add middleware layers
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
