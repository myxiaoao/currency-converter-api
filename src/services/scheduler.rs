use crate::services::{EcbFetcher, RedisStore};
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct RateScheduler {
    scheduler: JobScheduler,
}

impl RateScheduler {
    /// Create a new scheduler for updating exchange rates
    pub async fn new(
        cron_expression: String,
        fetcher: EcbFetcher,
        store: RedisStore,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::new().await?;

        // Create the scheduled job
        let job = Job::new_async(cron_expression.as_str(), move |_uuid, _lock| {
            let fetcher = fetcher.clone();
            let store = store.clone();

            Box::pin(async move {
                tracing::info!("Starting scheduled exchange rate update");

                match update_rates(&fetcher, &store).await {
                    Ok(_) => {
                        tracing::info!("Successfully completed scheduled exchange rate update");
                    }
                    Err(e) => {
                        tracing::error!("Scheduled update failed: {}", e);
                    }
                }
            })
        })?;

        scheduler.add(job).await?;

        Ok(Self { scheduler })
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting rate update scheduler");
        self.scheduler.start().await?;
        Ok(())
    }

    /// Stop the scheduler
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Shutting down rate update scheduler");
        self.scheduler.shutdown().await?;
        Ok(())
    }
}

/// Perform an immediate update of exchange rates (used for initial fetch and scheduled updates)
pub async fn update_rates(
    fetcher: &EcbFetcher,
    store: &RedisStore,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Fetching latest exchange rates from ECB");

    let rates = fetcher.fetch_rates().await?;

    tracing::info!(
        "Fetched {} exchange rates for {}",
        rates.rates.len(),
        rates.date
    );

    store.store_rates(&rates).await?;

    tracing::info!("Exchange rates updated successfully");

    Ok(())
}

// Make EcbFetcher cloneable for async job
impl Clone for EcbFetcher {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            ecb_url: self.ecb_url.clone(),
        }
    }
}
