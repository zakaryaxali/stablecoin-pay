use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::ApyRate;
use crate::error::AppError;

pub struct ApyRateRepository;

impl ApyRateRepository {
    pub async fn insert(
        pool: &PgPool,
        platform: &str,
        chain: &str,
        token: &str,
        apy_total: Decimal,
        apy_base: Option<Decimal>,
        apy_reward: Option<Decimal>,
        tvl_usd: Option<Decimal>,
        pool_id: Option<&str>,
        source: &str,
    ) -> Result<ApyRate, AppError> {
        let rate = sqlx::query_as::<_, ApyRate>(
            r#"
            INSERT INTO apy_rates (platform, chain, token, apy_total, apy_base, apy_reward, tvl_usd, pool_id, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(platform)
        .bind(chain)
        .bind(token)
        .bind(apy_total)
        .bind(apy_base)
        .bind(apy_reward)
        .bind(tvl_usd)
        .bind(pool_id)
        .bind(source)
        .fetch_one(pool)
        .await?;

        Ok(rate)
    }

    pub async fn get_latest_rates(
        pool: &PgPool,
        chain: &str,
        token: &str,
    ) -> Result<Vec<ApyRate>, AppError> {
        let rates = sqlx::query_as::<_, ApyRate>(
            r#"
            SELECT DISTINCT ON (platform) *
            FROM apy_rates
            WHERE chain = $1 AND token = $2
            ORDER BY platform, fetched_at DESC
            "#,
        )
        .bind(chain)
        .bind(token)
        .fetch_all(pool)
        .await?;

        Ok(rates)
    }

    pub async fn get_best_rate(
        pool: &PgPool,
        chain: &str,
        token: &str,
    ) -> Result<Option<ApyRate>, AppError> {
        let rate = sqlx::query_as::<_, ApyRate>(
            r#"
            SELECT DISTINCT ON (platform) *
            FROM apy_rates
            WHERE chain = $1 AND token = $2
            ORDER BY platform, fetched_at DESC
            "#,
        )
        .bind(chain)
        .bind(token)
        .fetch_all(pool)
        .await?
        .into_iter()
        .max_by(|a, b| a.apy_total.cmp(&b.apy_total));

        Ok(rate)
    }

    pub async fn get_history(
        pool: &PgPool,
        platform: &str,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<ApyRate>, AppError> {
        let rates = sqlx::query_as::<_, ApyRate>(
            r#"
            SELECT * FROM apy_rates
            WHERE platform = $1 AND fetched_at >= $2
            ORDER BY fetched_at DESC
            LIMIT $3
            "#,
        )
        .bind(platform)
        .bind(since)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(rates)
    }

    pub async fn cleanup_old_rates(
        pool: &PgPool,
        older_than: DateTime<Utc>,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(
            "DELETE FROM apy_rates WHERE created_at < $1",
        )
        .bind(older_than)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
