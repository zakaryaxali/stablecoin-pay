use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::{Transaction, TransactionStatus, TransactionType, WebhookEvent};
use crate::error::AppError;
use crate::repository::{TransactionRepository, WalletRepository, WebhookEventRepository};
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

// Webhook events query params
#[derive(Debug, Deserialize)]
pub struct WebhookEventsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Webhook events response
#[derive(Debug, Serialize)]
pub struct WebhookEventsResponse {
    pub events: Vec<WebhookEvent>,
    pub count: usize,
}

pub async fn get_webhook_events(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
    Query(query): Query<WebhookEventsQuery>,
) -> Result<Json<WebhookEventsResponse>, AppError> {
    // Validate address
    crate::services::solana::SolanaClient::validate_address(&address)?;

    // Check if wallet exists
    let wallet = WalletRepository::find_by_address(&state.db.pool, &address).await?;
    if wallet.is_none() {
        return Err(AppError::NotFound(format!("Wallet {} not found", address)));
    }

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let events =
        WebhookEventRepository::find_by_wallet(&state.db.pool, &address, limit, offset).await?;
    let count = events.len();

    Ok(Json(WebhookEventsResponse { events, count }))
}

// Test webhook response
#[derive(Debug, Serialize)]
pub struct TestWebhookResponse {
    pub success: bool,
    pub message: String,
}

pub async fn test_webhook(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Json<TestWebhookResponse>, AppError> {
    // Validate address
    crate::services::solana::SolanaClient::validate_address(&address)?;

    // Get wallet
    let wallet = WalletRepository::find_by_address(&state.db.pool, &address)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Wallet {} not found", address)))?;

    // Check if webhook URL is configured
    if wallet.webhook_url.is_none() {
        return Err(AppError::BadRequest(
            "No webhook URL configured for this wallet".into(),
        ));
    }

    // Send test webhook
    match state.webhook.send_test_webhook(&wallet).await {
        Ok(()) => Ok(Json(TestWebhookResponse {
            success: true,
            message: "Test webhook delivered successfully".into(),
        })),
        Err(e) => Ok(Json(TestWebhookResponse {
            success: false,
            message: format!("Webhook delivery failed: {}", e),
        })),
    }
}

// Detailed health response
#[derive(Debug, Serialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub database: HealthStatus,
    pub solana_rpc: HealthStatus,
    pub background_sync: BackgroundSyncStatus,
    pub webhooks: WebhookHealthStats,
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BackgroundSyncStatus {
    pub running: bool,
    pub last_sync: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WebhookHealthStats {
    pub pending: i64,
    pub delivered: i64,
    pub failed: i64,
}

pub async fn detailed_health(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DetailedHealthResponse>, AppError> {
    // Check database
    let db_status = match sqlx::query("SELECT 1")
        .execute(&state.db.pool)
        .await
    {
        Ok(_) => HealthStatus {
            status: "healthy".into(),
            message: None,
        },
        Err(e) => HealthStatus {
            status: "unhealthy".into(),
            message: Some(e.to_string()),
        },
    };

    // Check Solana RPC by fetching a known account
    let solana_status = match state
        .solana
        .get_usdc_balance("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v") // USDC mint address
        .await
    {
        Ok(_) => HealthStatus {
            status: "healthy".into(),
            message: None,
        },
        Err(e) => HealthStatus {
            status: "unhealthy".into(),
            message: Some(e.to_string()),
        },
    };

    // Get webhook stats
    let webhook_stats = state.webhook.get_stats().await?;

    let overall_status = if db_status.status == "healthy" && solana_status.status == "healthy" {
        "healthy"
    } else {
        "degraded"
    };

    Ok(Json(DetailedHealthResponse {
        status: overall_status.into(),
        database: db_status,
        solana_rpc: solana_status,
        background_sync: BackgroundSyncStatus {
            running: true, // Background sync is always running if server is up
            last_sync: None, // Could track this in the future
        },
        webhooks: WebhookHealthStats {
            pending: webhook_stats.pending,
            delivered: webhook_stats.delivered,
            failed: webhook_stats.failed,
        },
    }))
}
