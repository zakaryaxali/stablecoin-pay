use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Serialize;

use crate::error::AppError;
use crate::AppState;

// Health check
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok"
    }))
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

    // Check Solana RPC by getting current slot
    let solana_status = match state.solana.get_slot().await {
        Ok(slot) => HealthStatus {
            status: "healthy".into(),
            message: Some(format!("slot: {}", slot)),
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
