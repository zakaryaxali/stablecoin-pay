use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use stablecoin_pay::api;
use stablecoin_pay::config::Config;
use stablecoin_pay::db::Database;
use stablecoin_pay::services::apy::ApyService;
use stablecoin_pay::services::deposit::DepositService;
use stablecoin_pay::services::solana::SolanaClient;
use stablecoin_pay::services::sync::SyncService;
use stablecoin_pay::services::webhook::WebhookService;
use stablecoin_pay::AppState;

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

    // Initialize deposit service
    let deposit = Arc::new(DepositService::new(&config.solana_rpc_url));

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
        deposit,
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
