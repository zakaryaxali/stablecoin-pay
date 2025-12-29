use std::env;

use anyhow::{Context, Result};

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
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
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
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}
