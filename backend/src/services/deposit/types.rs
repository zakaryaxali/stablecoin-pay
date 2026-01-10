//! Types for deposit/withdraw operations.

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// ─────────────────────────────────────────────────────────────────────────────
// RPC Response Types
// ─────────────────────────────────────────────────────────────────────────────

/// Account info response value
#[derive(Deserialize)]
pub struct AccountValue {
    pub data: (String, String), // (base64_data, encoding)
    pub owner: String,          // Program that owns this account
}

/// Account info result wrapper
#[derive(Deserialize)]
pub struct AccountResult {
    pub value: Option<AccountValue>,
}

/// Blockhash response value
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockhashValue {
    pub blockhash: String,
    pub last_valid_block_height: u64,
}

/// Blockhash result wrapper
#[derive(Deserialize)]
pub struct BlockhashResult {
    pub value: BlockhashValue,
}

/// Signature status for transaction confirmation
#[derive(Deserialize, Debug)]
pub struct SignatureStatus {
    #[allow(dead_code)]
    pub slot: Option<u64>,
    #[allow(dead_code)]
    pub confirmations: Option<u64>,
    pub err: Option<serde_json::Value>,
    #[serde(rename = "confirmationStatus")]
    pub confirmation_status: Option<String>,
}

/// Signature status result wrapper
#[derive(Deserialize)]
pub struct StatusResult {
    pub value: Vec<Option<SignatureStatus>>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Protocol Data Types
// ─────────────────────────────────────────────────────────────────────────────

/// Build deposit transaction response
#[derive(Debug, Serialize)]
pub struct BuildDepositResponse {
    pub transaction: String, // base64 encoded unsigned transaction
    pub blockhash: String,
    pub last_valid_block_height: u64,
    pub protocol: String,
    pub amount_lamports: u64,
}

/// Kamino reserve data (fetched from on-chain)
#[derive(Debug, Clone)]
pub struct KaminoReserveData {
    pub reserve_address: Pubkey,
    pub liquidity_mint: Pubkey,
    pub liquidity_supply: Pubkey,
    pub collateral_mint: Pubkey,
    #[allow(dead_code)]
    pub collateral_supply: Pubkey,
    pub liquidity_token_program: Pubkey,
    pub collateral_token_program: Pubkey,
}

/// Save (Solend) reserve data (fetched from on-chain)
#[derive(Debug, Clone)]
pub struct SaveReserveData {
    pub reserve_address: Pubkey,
    pub liquidity_mint: Pubkey,
    pub liquidity_supply: Pubkey,
    pub collateral_mint: Pubkey,
    #[allow(dead_code)]
    pub collateral_supply: Pubkey,
    pub collateral_token_program: Pubkey,
}
