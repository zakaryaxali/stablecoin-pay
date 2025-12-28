use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::domain::{PaymentReceivedPayload, Transaction, Wallet, WebhookPayload, WebhookStatus};
use crate::error::AppError;
use crate::repository::WebhookEventRepository;

type HmacSha256 = Hmac<Sha256>;

/// Retry delays for webhook delivery (exponential backoff)
const RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(1),
    Duration::from_secs(5),
    Duration::from_secs(30),
];

/// Maximum number of delivery attempts before marking as failed
const MAX_ATTEMPTS: i32 = 3;

pub struct WebhookService {
    client: Client,
    pool: PgPool,
    webhook_secret: String,
}

impl WebhookService {
    pub fn new(pool: PgPool, webhook_secret: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            pool,
            webhook_secret,
        }
    }

    /// Sign a payload using HMAC-SHA256
    fn sign_payload(&self, payload: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload);
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Create a webhook event for a new transaction and attempt delivery
    pub async fn notify_payment_received(
        &self,
        wallet: &Wallet,
        transaction: &Transaction,
    ) -> Result<(), AppError> {
        // Check if we already have a webhook event for this transaction
        if WebhookEventRepository::exists_for_transaction(&self.pool, &transaction.signature).await? {
            info!(
                signature = %transaction.signature,
                "Webhook event already exists for transaction, skipping"
            );
            return Ok(());
        }

        // Skip if wallet has no webhook URL configured
        let webhook_url = match &wallet.webhook_url {
            Some(url) if !url.is_empty() => url.clone(),
            _ => {
                info!(
                    wallet = %wallet.address,
                    "No webhook URL configured for wallet, skipping notification"
                );
                return Ok(());
            }
        };

        // Build the payload
        let payment_data = PaymentReceivedPayload {
            signature: transaction.signature.clone(),
            wallet_address: transaction.wallet_address.clone(),
            amount: transaction.amount.to_string(),
            token: "USDC".to_string(),
            counterparty: transaction.counterparty.clone(),
            block_time: transaction.block_time,
        };

        let payload = WebhookPayload {
            event: "payment.received".to_string(),
            timestamp: Utc::now(),
            data: serde_json::to_value(&payment_data)?,
        };

        let payload_json = serde_json::to_value(&payload)?;

        // Create the webhook event record
        let event = WebhookEventRepository::create(
            &self.pool,
            &wallet.address,
            Some(&transaction.signature),
            "payment.received",
            payload_json.clone(),
        )
        .await?;

        info!(
            event_id = %event.id,
            wallet = %wallet.address,
            signature = %transaction.signature,
            "Created webhook event for payment.received"
        );

        // Attempt delivery
        self.deliver_webhook(&webhook_url, event.id, &payload_json)
            .await
    }

    /// Attempt to deliver a webhook with retry logic
    async fn deliver_webhook(
        &self,
        url: &str,
        event_id: sqlx::types::Uuid,
        payload: &serde_json::Value,
    ) -> Result<(), AppError> {
        let payload_bytes = serde_json::to_vec(payload)?;
        let signature = self.sign_payload(&payload_bytes);

        for (attempt, delay) in RETRY_DELAYS.iter().enumerate() {
            let attempt_num = attempt as i32 + 1;

            match self.send_webhook(url, &payload_bytes, &signature).await {
                Ok(()) => {
                    WebhookEventRepository::mark_delivered(&self.pool, event_id).await?;
                    info!(
                        event_id = %event_id,
                        attempt = attempt_num,
                        "Webhook delivered successfully"
                    );
                    return Ok(());
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    warn!(
                        event_id = %event_id,
                        attempt = attempt_num,
                        error = %error_msg,
                        "Webhook delivery failed"
                    );

                    // Update the event with attempt info
                    WebhookEventRepository::increment_attempt(&self.pool, event_id, Some(&error_msg))
                        .await?;

                    // If we've exhausted retries, mark as failed
                    if attempt_num >= MAX_ATTEMPTS as i32 {
                        WebhookEventRepository::mark_failed(&self.pool, event_id, &error_msg).await?;
                        error!(
                            event_id = %event_id,
                            "Webhook delivery failed after {} attempts",
                            MAX_ATTEMPTS
                        );
                        return Err(AppError::WebhookDeliveryFailed(error_msg));
                    }

                    // Wait before retrying
                    tokio::time::sleep(*delay).await;
                }
            }
        }

        Ok(())
    }

    /// Send a single webhook HTTP request
    async fn send_webhook(
        &self,
        url: &str,
        payload: &[u8],
        signature: &str,
    ) -> Result<(), AppError> {
        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", format!("sha256={}", signature))
            .body(payload.to_vec())
            .send()
            .await
            .map_err(|e| AppError::WebhookDeliveryFailed(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::WebhookDeliveryFailed(format!(
                "HTTP {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )))
        }
    }

    /// Retry all pending webhook events (for background job)
    pub async fn retry_pending_webhooks(&self) -> Result<u32, AppError> {
        let pending = WebhookEventRepository::find_pending(&self.pool, 100).await?;
        let mut retried = 0;

        for event in pending {
            // Skip events that have exceeded max attempts
            if event.attempts >= MAX_ATTEMPTS {
                WebhookEventRepository::mark_failed(
                    &self.pool,
                    event.id,
                    "Max retry attempts exceeded",
                )
                .await?;
                continue;
            }

            // Get the wallet to get the webhook URL
            let wallet = sqlx::query_as::<_, Wallet>(
                "SELECT * FROM wallets WHERE address = $1"
            )
            .bind(&event.wallet_address)
            .fetch_optional(&self.pool)
            .await?;

            let webhook_url = match wallet.and_then(|w| w.webhook_url) {
                Some(url) => url,
                None => {
                    WebhookEventRepository::mark_failed(
                        &self.pool,
                        event.id,
                        "Wallet webhook URL no longer configured",
                    )
                    .await?;
                    continue;
                }
            };

            // Attempt delivery (single attempt, not full retry loop)
            let payload_bytes = serde_json::to_vec(&event.payload)?;
            let signature = self.sign_payload(&payload_bytes);

            match self.send_webhook(&webhook_url, &payload_bytes, &signature).await {
                Ok(()) => {
                    WebhookEventRepository::mark_delivered(&self.pool, event.id).await?;
                    retried += 1;
                    info!(event_id = %event.id, "Pending webhook delivered on retry");
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let updated = WebhookEventRepository::increment_attempt(
                        &self.pool,
                        event.id,
                        Some(&error_msg),
                    )
                    .await?;

                    if updated.attempts >= MAX_ATTEMPTS {
                        WebhookEventRepository::mark_failed(&self.pool, event.id, &error_msg)
                            .await?;
                    }
                }
            }
        }

        Ok(retried)
    }

    /// Send a test webhook to verify URL is working
    pub async fn send_test_webhook(&self, wallet: &Wallet) -> Result<(), AppError> {
        let webhook_url = wallet
            .webhook_url
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("No webhook URL configured".into()))?;

        let payload = WebhookPayload {
            event: "test".to_string(),
            timestamp: Utc::now(),
            data: serde_json::json!({
                "message": "This is a test webhook",
                "wallet_address": wallet.address
            }),
        };

        let payload_json = serde_json::to_value(&payload)?;

        // Create event record for test webhook
        let event = WebhookEventRepository::create(
            &self.pool,
            &wallet.address,
            None, // No transaction for test webhooks
            "test",
            payload_json.clone(),
        )
        .await?;

        // Attempt single delivery (no retries for test)
        let payload_bytes = serde_json::to_vec(&payload)?;
        let signature = self.sign_payload(&payload_bytes);

        match self.send_webhook(webhook_url, &payload_bytes, &signature).await {
            Ok(()) => {
                WebhookEventRepository::mark_delivered(&self.pool, event.id).await?;
                info!(wallet = %wallet.address, "Test webhook delivered successfully");
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                WebhookEventRepository::mark_failed(&self.pool, event.id, &error_msg).await?;
                Err(e)
            }
        }
    }

    /// Get webhook delivery statistics
    pub async fn get_stats(&self) -> Result<WebhookStats, AppError> {
        let pending = WebhookEventRepository::count_by_status(&self.pool, WebhookStatus::Pending).await?;
        let delivered = WebhookEventRepository::count_by_status(&self.pool, WebhookStatus::Delivered).await?;
        let failed = WebhookEventRepository::count_by_status(&self.pool, WebhookStatus::Failed).await?;

        Ok(WebhookStats {
            pending,
            delivered,
            failed,
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct WebhookStats {
    pub pending: i64,
    pub delivered: i64,
    pub failed: i64,
}
