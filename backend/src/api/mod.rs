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
        .route("/wallets", post(handlers::create_wallet))
        .route("/wallets/:address/balance", get(handlers::get_balance))
        .route("/wallets/:address/transactions", get(handlers::get_transactions))
        .with_state(state)
}
