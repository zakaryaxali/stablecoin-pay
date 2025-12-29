use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

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

        let data: DefiLlamaResponse = response
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse DeFiLlama response: {}", e)))?;

        let target_projects = ["kamino-lend", "save", "marginfi-lend"];

        let rates: Vec<PoolRate> = data
            .data
            .into_iter()
            .filter(|pool| {
                pool.chain.to_lowercase() == "solana"
                    && pool.symbol.to_uppercase() == "USDC"
                    && target_projects.contains(&pool.project.as_str())
            })
            .filter_map(|pool| self.convert_pool_to_rate(pool))
            .collect();

        tracing::info!("Fetched {} USDC rates from DeFiLlama", rates.len());
        Ok(rates)
    }

    fn convert_pool_to_rate(&self, pool: DefiLlamaPool) -> Option<PoolRate> {
        let apy = pool.apy?;

        let platform = match pool.project.as_str() {
            "kamino-lend" => "kamino",
            "save" => "save",
            "marginfi-lend" => "marginfi",
            _ => return None,
        };

        Some(PoolRate {
            platform: platform.to_string(),
            chain: "solana".to_string(),
            token: "USDC".to_string(),
            apy_total: Decimal::from_str(&format!("{:.4}", apy)).unwrap_or_default(),
            apy_base: pool.apy_base.map(|v| Decimal::from_str(&format!("{:.4}", v)).unwrap_or_default()),
            apy_reward: pool.apy_reward.map(|v| Decimal::from_str(&format!("{:.4}", v)).unwrap_or_default()),
            tvl_usd: pool.tvl_usd.map(|v| Decimal::from_str(&format!("{:.2}", v)).unwrap_or_default()),
            pool_id: pool.pool,
        })
    }
}

impl Default for DefiLlamaClient {
    fn default() -> Self {
        Self::new()
    }
}
