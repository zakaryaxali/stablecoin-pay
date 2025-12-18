use sqlx::PgPool;

use crate::domain::Wallet;
use crate::error::AppError;

pub struct WalletRepository;

impl WalletRepository {
    pub async fn create(pool: &PgPool, address: &str, webhook_url: Option<&str>) -> Result<Wallet, AppError> {
        let wallet = sqlx::query_as::<_, Wallet>(
            r#"
            INSERT INTO wallets (address, webhook_url)
            VALUES ($1, $2)
            ON CONFLICT (address) DO UPDATE SET webhook_url = COALESCE($2, wallets.webhook_url)
            RETURNING *
            "#,
        )
        .bind(address)
        .bind(webhook_url)
        .fetch_one(pool)
        .await?;

        Ok(wallet)
    }

    pub async fn find_by_address(pool: &PgPool, address: &str) -> Result<Option<Wallet>, AppError> {
        let wallet = sqlx::query_as::<_, Wallet>(
            "SELECT * FROM wallets WHERE address = $1",
        )
        .bind(address)
        .fetch_optional(pool)
        .await?;

        Ok(wallet)
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<Wallet>, AppError> {
        let wallets = sqlx::query_as::<_, Wallet>(
            "SELECT * FROM wallets ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(wallets)
    }

    pub async fn delete(pool: &PgPool, address: &str) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM wallets WHERE address = $1")
            .bind(address)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
