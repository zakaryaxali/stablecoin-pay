use std::env;

use anyhow::{Context, Result};
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub solana_rpc_url: String,
    pub usdc_mint: String,
    pub port: u16,
    pub webhook_secret: String,
    pub environment: Environment,

    // --- Autonomous payment agent ---
    /// Anthropic API key for the in-loop Claude decision call (agent only).
    pub anthropic_api_key: Option<String>,
    /// Claude model used for settlement decisions.
    pub anthropic_model: String,
    /// Treasury secret key (base58) or path to a Solana CLI keypair file.
    pub agent_keypair: Option<String>,
    /// Faucet secret key (base58) or keypair file, used to simulate inbound payments.
    pub faucet_keypair: Option<String>,
    /// Treasury wallet address the agent watches for inbound USDC.
    pub treasury_address: Option<String>,
    /// Hard cap on a single payout, enforced in Rust after Claude decides.
    pub max_payout_usdc: Decimal,
    /// Hard cap on cumulative daily payouts.
    pub daily_limit_usdc: Decimal,
    /// Seconds between agent loop iterations.
    pub agent_loop_interval_secs: u64,
    /// Allow the agent to run against a non-devnet RPC. Defaults to false.
    pub agent_allow_mainnet: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            solana_rpc_url: env::var("HELIUS_API_KEY")
                .map(|key| format!("https://mainnet.helius-rpc.com/?api-key={}", key))
                .or_else(|_| env::var("SOLANA_RPC_URL"))
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            usdc_mint: env::var("USDC_MINT")
                .unwrap_or_else(|_| "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("PORT must be a valid number")?,
            webhook_secret: env::var("WEBHOOK_SECRET")
                .unwrap_or_else(|_| "default-webhook-secret-change-in-production".to_string()),
            environment: match env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .to_lowercase()
                .as_str()
            {
                "production" | "prod" => Environment::Production,
                _ => Environment::Development,
            },

            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
            anthropic_model: env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-6".to_string()),
            agent_keypair: env::var("AGENT_KEYPAIR").ok(),
            faucet_keypair: env::var("FAUCET_KEYPAIR").ok(),
            treasury_address: env::var("TREASURY_ADDRESS").ok(),
            max_payout_usdc: env::var("MAX_PAYOUT_USDC")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("MAX_PAYOUT_USDC must be a decimal")?,
            daily_limit_usdc: env::var("DAILY_LIMIT_USDC")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .context("DAILY_LIMIT_USDC must be a decimal")?,
            agent_loop_interval_secs: env::var("AGENT_LOOP_INTERVAL_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("AGENT_LOOP_INTERVAL_SECS must be a number")?,
            agent_allow_mainnet: env::var("AGENT_ALLOW_MAINNET")
                .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes"))
                .unwrap_or(false),
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}
