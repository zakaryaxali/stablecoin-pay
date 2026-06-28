//! Autonomous payment agent process entrypoint.
//!
//! Runs the observe → decide → act → verify loop in its own process, sharing
//! the backend crate (`stablecoin_pay`) with the API server. The API still
//! starts via `cargo run` (default-run); the agent runs via
//! `cargo run --bin agent`.
//!
//! Phase 1 is a scaffold: it wires up config + the cluster safety guard and
//! supports a `--once` flag that runs a single (stub) iteration and exits
//! non-zero on failure, so chain-touching phases get a scriptable signal.

use anyhow::Context;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use stablecoin_pay::agent::signer;
use stablecoin_pay::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stablecoin_pay=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    let run_once = std::env::args().any(|arg| arg == "--once");

    // Safety rail: the agent custodies a key and signs autonomously, so refuse
    // to start against a non-devnet RPC unless explicitly allowed.
    signer::ensure_safe_cluster(&config.solana_rpc_url, config.agent_allow_mainnet)
        .context("cluster safety check failed")?;
    tracing::info!(rpc = %config.solana_rpc_url, "cluster safety check passed");

    // Load the treasury keypair if configured (never log the secret).
    if let Some(source) = config.agent_keypair.as_deref() {
        use solana_sdk::signature::Signer;
        let keypair = signer::load_treasury_keypair(source).context("loading agent keypair")?;
        tracing::info!(treasury = %keypair.pubkey(), "treasury keypair loaded");
    } else {
        tracing::warn!("AGENT_KEYPAIR not set; agent cannot sign payouts yet");
    }

    if run_once {
        tracing::info!("running a single agent iteration (Phase 1 stub)");
        // Phase 3+ wires the observe → decide → act → verify loop here.
        tracing::info!("single iteration complete");
        return Ok(());
    }

    tracing::info!(
        interval_secs = config.agent_loop_interval_secs,
        "agent loop not yet implemented (Phase 1 stub); exiting"
    );
    Ok(())
}
