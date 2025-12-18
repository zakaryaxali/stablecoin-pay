mod api;
mod config;
mod db;
mod domain;
mod error;
mod repository;
mod services;

use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::Database;
use crate::services::solana::SolanaClient;

pub struct AppState {
    pub db: Database,
    pub solana: SolanaClient,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stablecoin_pay=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    tracing::info!("Starting server on port {}", config.port);

    // Initialize database
    let db = Database::connect(&config.database_url).await?;
    db.run_migrations().await?;

    // Initialize Solana client
    let solana = SolanaClient::new(&config.solana_rpc_url, &config.usdc_mint);

    // Create app state
    let state = Arc::new(AppState { db, solana, config });

    // Build router
    let app = Router::new()
        .merge(api::routes(state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Start server
    let addr = format!("0.0.0.0:{}", state.config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
