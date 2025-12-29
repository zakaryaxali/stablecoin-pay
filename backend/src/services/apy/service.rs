use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{Duration, Utc};
use sqlx::PgPool;
use tokio::task::JoinHandle;

use crate::domain::{ApyRate, ApyRateResponse};
use crate::error::AppError;
use crate::repository::ApyRateRepository;

use super::defillama::DefiLlamaClient;

pub struct ApyService {
    pool: PgPool,
    defillama: DefiLlamaClient,
    shutdown: Arc<AtomicBool>,
}

impl ApyService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            defillama: DefiLlamaClient::new(),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_background_fetch(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            tracing::info!("Starting APY background fetch service");

            // Initial fetch
            if let Err(e) = self.fetch_and_store_rates().await {
                tracing::error!("Initial APY fetch failed: {}", e);
            }

            // Fetch every 5 minutes
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            interval.tick().await; // Skip the first immediate tick

            loop {
                if self.shutdown.load(Ordering::SeqCst) {
                    tracing::info!("APY fetch service shutting down");
                    break;
                }

                interval.tick().await;

                if let Err(e) = self.fetch_and_store_rates().await {
                    tracing::error!("APY fetch failed: {}", e);
                }

                // Cleanup old rates (keep 7 days of history)
                let cutoff = Utc::now() - Duration::days(7);
                if let Err(e) = ApyRateRepository::cleanup_old_rates(&self.pool, cutoff).await {
                    tracing::error!("Failed to cleanup old APY rates: {}", e);
                }
            }
        })
    }

    pub async fn fetch_and_store_rates(&self) -> Result<Vec<ApyRate>, AppError> {
        tracing::info!("Fetching APY rates from DeFiLlama");

        let rates = self.defillama.fetch_solana_usdc_rates().await?;

        let mut stored_rates = Vec::new();

        for rate in rates {
            match ApyRateRepository::insert(
                &self.pool,
                &rate.platform,
                &rate.chain,
                &rate.token,
                rate.apy_total,
                rate.apy_base,
                rate.apy_reward,
                rate.tvl_usd,
                Some(&rate.pool_id),
                "defillama",
            )
            .await
            {
                Ok(stored) => {
                    tracing::debug!(
                        "Stored APY rate for {}: {}%",
                        stored.platform,
                        stored.apy_total
                    );
                    stored_rates.push(stored);
                }
                Err(e) => {
                    tracing::error!("Failed to store APY rate for {}: {}", rate.platform, e);
                }
            }
        }

        tracing::info!("Stored {} APY rates", stored_rates.len());
        Ok(stored_rates)
    }

    pub async fn get_latest_rates(&self) -> Result<Vec<ApyRateResponse>, AppError> {
        let rates = ApyRateRepository::get_latest_rates(&self.pool, "solana", "USDC").await?;
        Ok(rates.into_iter().map(ApyRateResponse::from).collect())
    }

    pub async fn get_best_rate(&self) -> Result<Option<ApyRateResponse>, AppError> {
        let rate = ApyRateRepository::get_best_rate(&self.pool, "solana", "USDC").await?;
        Ok(rate.map(ApyRateResponse::from))
    }

    pub async fn get_history(
        &self,
        platform: &str,
        hours: i64,
    ) -> Result<Vec<ApyRateResponse>, AppError> {
        let since = Utc::now() - Duration::hours(hours);
        let rates = ApyRateRepository::get_history(&self.pool, platform, since, 1000).await?;
        Ok(rates.into_iter().map(ApyRateResponse::from).collect())
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}
