use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

use super::protocols::{defillama_project_names, find_by_defillama_name};
use crate::error::AppError;

const DEFILLAMA_YIELDS_URL: &str = "https://yields.llama.fi/pools";

#[derive(Debug, Deserialize)]
pub struct DefiLlamaResponse {
    pub status: String,
    pub data: Vec<DefiLlamaPool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefiLlamaPool {
    pub chain: String,
    pub project: String,
    pub symbol: String,
    pub tvl_usd: Option<f64>,
    pub apy: Option<f64>,
    pub apy_base: Option<f64>,
    pub apy_reward: Option<f64>,
    pub pool: String,
    pub pool_meta: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PoolRate {
    pub platform: String,
    pub chain: String,
    pub token: String,
    pub apy_total: Decimal,
    pub apy_base: Option<Decimal>,
    pub apy_reward: Option<Decimal>,
    pub tvl_usd: Option<Decimal>,
    pub pool_id: String,
    pub pool_meta: Option<String>,
}

pub struct DefiLlamaClient {
    client: Client,
}

impl DefiLlamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_solana_usdc_rates(&self) -> Result<Vec<PoolRate>, AppError> {
        let response = self
            .client
            .get(DEFILLAMA_YIELDS_URL)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Failed to fetch DeFiLlama data: {}", e)))?;

        let data: DefiLlamaResponse = response.json().await.map_err(|e| {
            AppError::External(format!("Failed to parse DeFiLlama response: {}", e))
        })?;

        let target_projects = defillama_project_names();

        let rates: Vec<PoolRate> = data
            .data
            .into_iter()
            .filter(|pool| {
                pool.chain.to_lowercase() == "solana"
                    && pool.symbol.to_uppercase() == "USDC"
                    && target_projects.contains(&pool.project.as_str())
            })
            // Apply protocol-specific pool filters (e.g., Save only uses Main Pool)
            .filter(|pool| {
                find_by_defillama_name(&pool.project)
                    .map(|config| config.matches_pool(pool.pool_meta.as_deref()))
                    .unwrap_or(false)
            })
            .filter_map(|pool| self.convert_pool_to_rate(pool))
            .collect();

        tracing::info!("Fetched {} USDC rates from DeFiLlama", rates.len());
        Ok(rates)
    }

    fn convert_pool_to_rate(&self, pool: DefiLlamaPool) -> Option<PoolRate> {
        let apy = pool.apy?;

        let config = find_by_defillama_name(&pool.project)?;

        Some(PoolRate {
            platform: config.internal_name.to_string(),
            chain: "solana".to_string(),
            token: "USDC".to_string(),
            apy_total: Decimal::from_str(&format!("{:.4}", apy)).unwrap_or_default(),
            apy_base: pool
                .apy_base
                .map(|v| Decimal::from_str(&format!("{:.4}", v)).unwrap_or_default()),
            apy_reward: pool
                .apy_reward
                .map(|v| Decimal::from_str(&format!("{:.4}", v)).unwrap_or_default()),
            tvl_usd: pool
                .tvl_usd
                .map(|v| Decimal::from_str(&format!("{:.2}", v)).unwrap_or_default()),
            pool_id: pool.pool,
            pool_meta: pool.pool_meta,
        })
    }
}

impl Default for DefiLlamaClient {
    fn default() -> Self {
        Self::new()
    }
}
