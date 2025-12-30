use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::WebhookEvent;
use crate::error::AppError;
use crate::repository::{WalletRepository, WebhookEventRepository};
use crate::AppState;

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
