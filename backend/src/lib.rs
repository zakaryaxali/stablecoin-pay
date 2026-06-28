//! Library root for the stablecoin payment platform.
//!
//! Both binaries — the API server (`src/main.rs`) and the autonomous payment
//! agent (`src/bin/agent.rs`) — depend on this crate so they can share modules,
//! `AppState`, config, errors, and the Solana client.

pub mod agent;
pub mod api;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod repository;
pub mod services;

use std::sync::Arc;

use crate::config::Config;
use crate::db::Database;
use crate::services::apy::ApyService;
use crate::services::deposit::DepositService;
use crate::services::solana::SolanaClient;
use crate::services::sync::SyncService;
use crate::services::webhook::WebhookService;

/// Shared application state for the API server.
///
/// Lives at the crate root so handlers can reference `crate::AppState` and the
/// binaries can construct it via `stablecoin_pay::AppState`.
pub struct AppState {
    pub db: Database,
    pub solana: Arc<SolanaClient>,
    pub webhook: Arc<WebhookService>,
    pub sync: Arc<SyncService>,
    pub apy: Arc<ApyService>,
    pub deposit: Arc<DepositService>,
    pub config: Config,
}
