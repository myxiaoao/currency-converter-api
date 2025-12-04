mod config;
mod error;
mod handlers;
mod models;
mod routes;
mod services;

use config::Config;
use routes::create_router;
use services::{EcbFetcher, RateScheduler, RedisStore, update_rates};
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,currency_converter_api=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Loaded configuration");

    // Connect to Redis
    let store = RedisStore::new(&config.redis_url).await?;
    tracing::info!("Connected to Redis");

    // Create ECB fetcher
    let fetcher = EcbFetcher::new(config.ecb_url.clone());

    // Perform initial fetch (non-blocking - log error but continue)
    tracing::info!("Attempting initial fetch of exchange rates...");
    match update_rates(&fetcher, &store).await {
        Ok(_) => {
            tracing::info!("Initial exchange rates loaded successfully");
        }
        Err(e) => {
            tracing::warn!("Initial fetch failed (will retry on schedule): {}", e);
        }
    }

    // Create and start the scheduler
    let mut scheduler =
        RateScheduler::new(config.update_cron.clone(), fetcher, store.clone()).await?;
    scheduler.start().await?;
    tracing::info!(
        "Rate update scheduler started with cron: {}",
        config.update_cron
    );

    // Create router with shared state
    let app = create_router(store);

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.server_address()).await?;
    let addr = config.server_address();
    tracing::info!("Server listening on {}", addr);

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Shutdown scheduler on exit
    tracing::info!("Shutting down scheduler...");
    scheduler.shutdown().await?;
    tracing::info!("Server shutdown complete");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal");
        },
    }
}
