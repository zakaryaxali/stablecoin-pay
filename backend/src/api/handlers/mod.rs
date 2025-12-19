use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::Transaction;
use crate::error::AppError;
use crate::repository::{TransactionRepository, WalletRepository};
use crate::AppState;

// Health check
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok"
    }))
}

// Create wallet request
#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub address: String,
    pub webhook_url: Option<String>,
}

// Create wallet response
#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub address: String,
    pub webhook_url: Option<String>,
    pub created_at: String,
}

pub async fn create_wallet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWalletRequest>,
) -> Result<Json<WalletResponse>, AppError> {
    // Validate address
    crate::services::solana::SolanaClient::validate_address(&req.address)?;

    let wallet = WalletRepository::create(
        &state.db.pool,
        &req.address,
        req.webhook_url.as_deref(),
    )
    .await?;

    Ok(Json(WalletResponse {
        address: wallet.address,
        webhook_url: wallet.webhook_url,
        created_at: wallet.created_at.to_rfc3339(),
    }))
}

// Balance response
#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub token: String,
    pub symbol: String,
    pub amount: String,
    pub usd_value: String,
}

pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Json<BalanceResponse>, AppError> {
    // Validate address
    crate::services::solana::SolanaClient::validate_address(&address)?;

    // Get balance from Solana
    let balance = state.solana.get_usdc_balance(&address).await?;

    Ok(Json(BalanceResponse {
        address,
        token: "USD Coin".to_string(),
        symbol: "USDC".to_string(),
        amount: balance.amount.to_string(),
        usd_value: balance.amount.to_string(), // USDC is 1:1 with USD
    }))
}

// Transactions query params
#[derive(Debug, Deserialize)]
pub struct TransactionsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Transactions response
#[derive(Debug, Serialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<Transaction>,
    pub count: usize,
}

pub async fn get_transactions(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
    Query(query): Query<TransactionsQuery>,
) -> Result<Json<TransactionsResponse>, AppError> {
    // Validate address
    crate::services::solana::SolanaClient::validate_address(&address)?;

    // Check if wallet is registered
    let wallet = WalletRepository::find_by_address(&state.db.pool, &address).await?;
    if wallet.is_none() {
        return Err(AppError::NotFound(format!(
            "Wallet {} not registered. POST /wallets to register it first.",
            address
        )));
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let transactions =
        TransactionRepository::find_by_wallet(&state.db.pool, &address, limit, offset).await?;

    let count = transactions.len();

    Ok(Json(TransactionsResponse {
        transactions,
        count,
    }))
}
