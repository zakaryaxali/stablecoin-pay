use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum WebhookStatus {
    Pending,
    Delivered,
    Failed,
}

impl std::fmt::Display for WebhookStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookStatus::Pending => write!(f, "pending"),
            WebhookStatus::Delivered => write!(f, "delivered"),
            WebhookStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub wallet_address: String,
    pub transaction_signature: Option<String>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: WebhookStatus,
    pub attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Payload structure for payment.received webhook events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentReceivedPayload {
    pub signature: String,
    pub wallet_address: String,
    pub amount: String,
    pub token: String,
    pub counterparty: String,
    pub block_time: DateTime<Utc>,
}

/// Full webhook event payload sent to webhook URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}
