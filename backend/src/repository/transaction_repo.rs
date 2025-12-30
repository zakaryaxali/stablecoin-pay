use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::{Transaction, TransactionStatus, TransactionType};
use crate::error::AppError;

pub struct TransactionRepository;

impl TransactionRepository {
    pub async fn create(
        pool: &PgPool,
        signature: &str,
        wallet_address: &str,
        tx_type: TransactionType,
        amount: Decimal,
        token_mint: &str,
        counterparty: &str,
        status: TransactionStatus,
        block_time: DateTime<Utc>,
    ) -> Result<Transaction, AppError> {
        let tx = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (signature, wallet_address, tx_type, amount, token_mint, counterparty, status, block_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (signature) DO NOTHING
            RETURNING *
            "#,
        )
        .bind(signature)
        .bind(wallet_address)
        .bind(tx_type.to_string())
        .bind(amount)
        .bind(token_mint)
        .bind(counterparty)
        .bind(status.to_string())
        .bind(block_time)
        .fetch_one(pool)
        .await?;

        Ok(tx)
    }

    pub async fn find_by_wallet(
        pool: &PgPool,
        wallet_address: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, AppError> {
        let txs = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE wallet_address = $1
            ORDER BY block_time DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(wallet_address)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(txs)
    }

    pub async fn exists(pool: &PgPool, signature: &str) -> Result<bool, AppError> {
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM transactions WHERE signature = $1)",
        )
        .bind(signature)
        .fetch_one(pool)
        .await?;

        Ok(exists.0)
    }
}
