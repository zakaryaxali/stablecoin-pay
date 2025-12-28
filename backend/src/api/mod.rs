mod handlers;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/health/detailed", get(handlers::detailed_health))
        .route("/wallets", post(handlers::create_wallet))
        .route("/wallets/:address/balance", get(handlers::get_balance))
        .route("/wallets/:address/transactions", get(handlers::get_transactions))
        .route("/wallets/:address/webhook-events", get(handlers::get_webhook_events))
        .route("/wallets/:address/webhook/test", post(handlers::test_webhook))
        .with_state(state)
}
