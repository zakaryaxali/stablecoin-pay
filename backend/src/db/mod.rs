use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        tracing::info!("Running database migrations...");
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        tracing::info!("Migrations complete");
        Ok(())
    }
}
