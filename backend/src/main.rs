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
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::Database;
use crate::services::apy::ApyService;
use crate::services::solana::SolanaClient;
use crate::services::sync::SyncService;
use crate::services::webhook::WebhookService;

pub struct AppState {
    pub db: Database,
    pub solana: Arc<SolanaClient>,
    pub webhook: Arc<WebhookService>,
    pub sync: Arc<SyncService>,
    pub apy: Arc<ApyService>,
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
    let solana = Arc::new(SolanaClient::new(&config.solana_rpc_url, &config.usdc_mint));

    // Initialize webhook service
    let webhook = Arc::new(WebhookService::new(
        db.pool.clone(),
        config.webhook_secret.clone(),
    ));

    // Initialize sync service
    let sync = Arc::new(SyncService::new(
        db.pool.clone(),
        solana.clone(),
        webhook.clone(),
    ));

    // Initialize APY service
    let apy = Arc::new(ApyService::new(db.pool.clone()));

    // Start background sync
    let sync_handle = sync.clone().start_background_sync();

    // Start APY background fetch
    let apy_handle = apy.clone().start_background_fetch();

    // Create app state
    let state = Arc::new(AppState {
        db,
        solana,
        webhook,
        sync: sync.clone(),
        apy: apy.clone(),
        config,
    });

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

    // Start server with graceful shutdown
    let addr = format!("0.0.0.0:{}", state.config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(sync, apy))
        .await?;

    // Wait for background tasks to finish
    sync_handle.abort();
    apy_handle.abort();
    tracing::info!("Server shutdown complete");

    Ok(())
}

async fn shutdown_signal(sync: Arc<SyncService>, apy: Arc<ApyService>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, stopping background services...");
    sync.shutdown();
    apy.shutdown();
}
