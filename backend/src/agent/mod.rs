//! Autonomous on-chain payment agent.
//!
//! The agent runs an observe → decide → act → verify loop: it watches a
//! treasury wallet for inbound USDC, asks Claude how to settle it against
//! stored instructions, validates the decision against hard policy guardrails,
//! then signs and sends the payouts itself.
//!
//! Phase 1 scaffolds the module boundary only:
//! - [`signer`] — load the treasury keypair and guard against non-devnet RPCs.
//! - [`types`]  — the decision/payout data model shared across the loop.
//!
//! Later phases add `context`, `claude`, `policy`, `executor`, and the
//! `AgentService` loop (mirroring [`crate::services::sync::SyncService`]).

pub mod signer;
pub mod types;
