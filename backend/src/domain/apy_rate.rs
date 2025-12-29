use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApyRate {
    pub id: Uuid,
    pub platform: String,
    pub chain: String,
    pub token: String,
    pub apy_total: Decimal,
    pub apy_base: Option<Decimal>,
    pub apy_reward: Option<Decimal>,
    pub tvl_usd: Option<Decimal>,
    pub pool_id: Option<String>,
    pub source: String,
    pub fetched_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApyRateResponse {
    pub platform: String,
    pub chain: String,
    pub token: String,
    pub apy_total: Decimal,
    pub apy_base: Option<Decimal>,
    pub apy_reward: Option<Decimal>,
    pub tvl_usd: Option<Decimal>,
    pub fetched_at: DateTime<Utc>,
}

impl From<ApyRate> for ApyRateResponse {
    fn from(rate: ApyRate) -> Self {
        Self {
            platform: rate.platform,
            chain: rate.chain,
            token: rate.token,
            apy_total: rate.apy_total,
            apy_base: rate.apy_base,
            apy_reward: rate.apy_reward,
            tvl_usd: rate.tvl_usd,
            fetched_at: rate.fetched_at,
        }
    }
}
