use chrono::Utc;
use sqlx::types::Uuid;
use sqlx::PgPool;

use crate::domain::{WebhookEvent, WebhookStatus};
use crate::error::AppError;

pub struct WebhookEventRepository;

impl WebhookEventRepository {
    pub async fn create(
        pool: &PgPool,
        wallet_address: &str,
        transaction_signature: Option<&str>,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<WebhookEvent, AppError> {
        let event = sqlx::query_as::<_, WebhookEvent>(
            r#"
            INSERT INTO webhook_events (wallet_address, transaction_signature, event_type, payload)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(wallet_address)
        .bind(transaction_signature)
        .bind(event_type)
        .bind(payload)
        .fetch_one(pool)
        .await?;

        Ok(event)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<WebhookEvent>, AppError> {
        let event = sqlx::query_as::<_, WebhookEvent>(
            "SELECT * FROM webhook_events WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(event)
    }

    pub async fn find_by_wallet(
        pool: &PgPool,
        wallet_address: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookEvent>, AppError> {
        let events = sqlx::query_as::<_, WebhookEvent>(
            r#"
            SELECT * FROM webhook_events
            WHERE wallet_address = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(wallet_address)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    pub async fn find_pending(pool: &PgPool, limit: i64) -> Result<Vec<WebhookEvent>, AppError> {
        let events = sqlx::query_as::<_, WebhookEvent>(
            r#"
            SELECT * FROM webhook_events
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    pub async fn mark_delivered(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE webhook_events
            SET status = 'delivered', delivered_at = $1, attempts = attempts + 1, last_attempt_at = $1
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn mark_failed(pool: &PgPool, id: Uuid, error: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE webhook_events
            SET status = 'failed', last_error = $1, attempts = attempts + 1, last_attempt_at = $2
            WHERE id = $3
            "#,
        )
        .bind(error)
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn increment_attempt(pool: &PgPool, id: Uuid, error: Option<&str>) -> Result<WebhookEvent, AppError> {
        let event = sqlx::query_as::<_, WebhookEvent>(
            r#"
            UPDATE webhook_events
            SET attempts = attempts + 1, last_attempt_at = $1, last_error = COALESCE($2, last_error)
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(Utc::now())
        .bind(error)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(event)
    }

    pub async fn count_by_status(pool: &PgPool, status: WebhookStatus) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM webhook_events WHERE status = $1",
        )
        .bind(status.to_string())
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    pub async fn exists_for_transaction(pool: &PgPool, transaction_signature: &str) -> Result<bool, AppError> {
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM webhook_events WHERE transaction_signature = $1)",
        )
        .bind(transaction_signature)
        .fetch_one(pool)
        .await?;

        Ok(exists.0)
    }
}
