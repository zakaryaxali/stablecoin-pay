//! Core data model for the autonomous payment agent.
//!
//! The invariant is "model proposes, Rust disposes": Claude emits exactly one
//! structured [`Decision`] per inbound payment, the policy gate validates it,
//! and the executor signs and sends. There is never a `send_transaction` tool
//! exposed to the model.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The outcome Claude commits to for a single inbound payment.
///
/// These map 1:1 to the decision tool schema exposed to the model: it may only
/// choose to pay recipients, hold, or alert — never to execute directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentAction {
    /// Split and pay the inbound funds to one or more recipients.
    PayRecipients,
    /// Take no action; record the reasoning for later review.
    Hold,
    /// Flag the payment for human attention.
    Alert,
}

/// A single payout leg: how much USDC to send to which address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payout {
    /// Recipient wallet (base58 Solana address).
    pub address: String,
    /// Amount of USDC to send (human units, 6 decimals on-chain).
    pub amount: Decimal,
}

/// The single structured decision Claude returns for an inbound payment.
///
/// Persisted (with raw `reasoning`) to `agent_decisions` for the activity feed
/// and audit trail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    /// The chosen action.
    pub action: AgentAction,
    /// Claude's natural-language justification, surfaced in the UI.
    pub reasoning: String,
    /// Payout legs — non-empty only when `action` is [`AgentAction::PayRecipients`].
    #[serde(default)]
    pub recipients: Vec<Payout>,
}

/// How an inbound payment is matched to a settlement instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchKind {
    /// Apply to the next inbound payment, whatever its sender.
    NextInbound,
    /// Apply only to inbound payments from a specific sender address.
    FromSender,
}

/// One recipient leg of a settlement instruction.
///
/// Exactly one of `share_bps` (proportional split) or `fixed_amount` (a fixed
/// USDC amount) is expected to be set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionRecipient {
    /// Recipient wallet (base58 Solana address).
    pub address: String,
    /// Share of the inbound amount in basis points (1/100th of a percent).
    #[serde(default)]
    pub share_bps: Option<u32>,
    /// Fixed USDC amount for this recipient.
    #[serde(default)]
    pub fixed_amount: Option<Decimal>,
}

/// A stored rule for how to settle inbound funds.
///
/// Mirrors the `settlement_instructions` table added in a later phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementInstruction {
    /// Human-readable label for the activity feed.
    pub label: String,
    /// How inbound payments are matched to this instruction.
    pub match_kind: MatchKind,
    /// Sender address to match on, when `match_kind` is [`MatchKind::FromSender`].
    #[serde(default)]
    pub match_value: Option<String>,
    /// Recipients and their split definitions.
    pub recipients: Vec<InstructionRecipient>,
    /// Fee taken off the top, in basis points.
    #[serde(default)]
    pub fee_bps: u32,
}
