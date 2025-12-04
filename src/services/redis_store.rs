use crate::error::ApiError;
use crate::models::DailyRate;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};

const RATES_KEY: &str = "exchange:rates:latest";
const DATE_KEY: &str = "exchange:rates:date";

#[derive(Clone)]
pub struct RedisStore {
    manager: ConnectionManager,
}

impl RedisStore {
    /// Create a new Redis store with connection manager
    pub async fn new(redis_url: &str) -> Result<Self, ApiError> {
        tracing::info!("Connecting to Redis at: {}", redis_url);

        let client = Client::open(redis_url).map_err(|e| ApiError::RedisError(e))?;

        let manager = ConnectionManager::new(client)
            .await
            .map_err(|e| ApiError::RedisError(e))?;

        tracing::info!("Successfully connected to Redis");

        Ok(Self { manager })
    }

    /// Store exchange rates in Redis
    pub async fn store_rates(&self, rates: &DailyRate) -> Result<(), ApiError> {
        let mut conn = self.manager.clone();

        // Serialize rates to JSON
        let json = serde_json::to_string(rates)
            .map_err(|e| ApiError::InternalError(format!("Failed to serialize rates: {}", e)))?;

        // Store both the rates and the date
        conn.set::<_, _, ()>(RATES_KEY, json).await?;
        conn.set::<_, _, ()>(DATE_KEY, &rates.date).await?;

        tracing::info!("Stored exchange rates for {} in Redis", rates.date);

        Ok(())
    }

    /// Retrieve exchange rates from Redis
    pub async fn get_rates(&self) -> Result<Option<DailyRate>, ApiError> {
        let mut conn = self.manager.clone();

        let json: Option<String> = conn.get(RATES_KEY).await?;

        match json {
            Some(data) => {
                let rates: DailyRate = serde_json::from_str(&data).map_err(|e| {
                    ApiError::InternalError(format!("Failed to deserialize rates: {}", e))
                })?;

                tracing::debug!("Retrieved exchange rates for {} from Redis", rates.date);
                Ok(Some(rates))
            }
            None => {
                tracing::warn!("No exchange rates found in Redis");
                Ok(None)
            }
        }
    }

    /// Get the date of last update
    pub async fn get_last_update_date(&self) -> Result<Option<String>, ApiError> {
        let mut conn = self.manager.clone();
        let date: Option<String> = conn.get(DATE_KEY).await?;
        Ok(date)
    }

    /// Health check for Redis connection
    pub async fn health_check(&self) -> Result<(), ApiError> {
        let mut conn = self.manager.clone();
        redis::cmd("PING")
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| ApiError::RedisError(e))?;
        Ok(())
    }
}
