use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::{Transaction, TransactionStatus, TransactionType};
use crate::error::AppError;
use crate::repository::{TransactionRepository, WalletRepository};
use crate::AppState;

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

    let wallet =
        WalletRepository::create(&state.db.pool, &req.address, req.webhook_url.as_deref()).await?;

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

    // Sync recent transactions from Solana before returning
    let sync_limit = 20; // Fetch last 20 signatures to check
    match state
        .solana
        .sync_wallet_transactions(&address, sync_limit)
        .await
    {
        Ok(parsed_txs) => {
            // Store each transaction (idempotent - ON CONFLICT DO NOTHING)
            for tx in parsed_txs {
                let tx_type = if tx.tx_type == "send" {
                    TransactionType::Send
                } else {
                    TransactionType::Receive
                };

                let _ = TransactionRepository::create(
                    &state.db.pool,
                    &tx.signature,
                    &tx.wallet_address,
                    tx_type,
                    tx.amount,
                    &tx.token_mint,
                    &tx.counterparty,
                    TransactionStatus::Confirmed,
                    tx.block_time,
                    tx.protocol.as_deref(),
                )
                .await;
            }
        }
        Err(e) => {
            // Log sync error but continue to return cached data
            tracing::warn!("Failed to sync transactions from Solana: {}", e);
        }
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

// Staked position for a single protocol
#[derive(Debug, Serialize)]
pub struct StakedPosition {
    pub protocol: String,
    pub token: String,
    pub symbol: String,
    pub amount: String,
    pub mint: String,
}

// Staked balance response with multiple positions
#[derive(Debug, Serialize)]
pub struct StakedBalanceResponse {
    pub address: String,
    pub positions: Vec<StakedPosition>,
}

pub async fn get_staked_balance(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Json<StakedBalanceResponse>, AppError> {
    // Validate address
    crate::services::solana::SolanaClient::validate_address(&address)?;

    let mut positions = Vec::new();

    // Get Kamino position (kToken)
    if let Ok(kamino_mint) = state.deposit.get_kamino_collateral_mint().await {
        if let Ok(balance) = state.solana.get_token_balance(&address, &kamino_mint).await {
            if !balance.amount.is_zero() {
                positions.push(StakedPosition {
                    protocol: "kamino".to_string(),
                    token: "Kamino USDC".to_string(),
                    symbol: "kUSDC".to_string(),
                    amount: balance.amount.to_string(),
                    mint: kamino_mint,
                });
            }
        }
    }

    // Get Save position (cToken) - we use the Main Pool reserve
    if let Ok(save_mint) = state.deposit.get_save_collateral_mint().await {
        if let Ok(balance) = state.solana.get_token_balance(&address, &save_mint).await {
            if !balance.amount.is_zero() {
                positions.push(StakedPosition {
                    protocol: "save".to_string(),
                    token: "Save USDC (Main Pool)".to_string(),
                    symbol: "cUSDC".to_string(),
                    amount: balance.amount.to_string(),
                    mint: save_mint,
                });
            }
        }
    }

    Ok(Json(StakedBalanceResponse { address, positions }))
}
