//! Deposit transaction builder for Kamino and Save protocols.
//!
//! Builds unsigned transactions that get signed by the user's wallet on the frontend.
//!
//! ## Architecture Decision
//!
//! This service serializes complete Transaction objects using bincode and sends them
//! as base64 to the frontend. The frontend deserializes and signs with the wallet.
//!
//! **Trade-off:** This approach couples backend (Rust solana-sdk) and frontend
//! (@solana/web3.js) serialization formats. For a POC this is acceptable.
//!
//! **Production alternative:** Return instruction parameters as JSON and let the
//! frontend build the Transaction natively. This avoids SDK version coupling and
//! gives the frontend flexibility to add instructions (e.g., create ATAs).

mod constants;
mod kamino;
mod rpc;
mod save;
mod types;

pub use types::BuildDepositResponse;

use reqwest::Client;
use sha2::{Digest, Sha256};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::str::FromStr;

use constants::{ASSOCIATED_TOKEN_PROGRAM, SYSTEM_PROGRAM};

/// Service for building deposit/withdraw transactions
pub struct DepositService {
    client: Client,
    rpc_url: String,
}

impl DepositService {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            client: Client::new(),
            rpc_url: rpc_url.to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────────────

/// Compute Anchor instruction discriminator: sha256("global:<name>")[0..8]
fn anchor_discriminator(name: &str) -> [u8; 8] {
    let preimage = format!("global:{}", name);
    let hash = Sha256::digest(preimage.as_bytes());
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash[..8]);
    discriminator
}

/// Derive Associated Token Account address
fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    let ata_program = Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM).unwrap();

    let (ata, _) = Pubkey::find_program_address(
        &[wallet.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ata_program,
    );
    ata
}

/// Build create_associated_token_account_idempotent instruction
fn build_create_ata_idempotent_instruction(
    payer: &Pubkey,
    wallet: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let ata_program = Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM).unwrap();
    let system_program = Pubkey::from_str(SYSTEM_PROGRAM).unwrap();
    let ata = get_associated_token_address(wallet, mint, token_program);

    // Instruction discriminator for create_idempotent is 1
    let data = vec![1u8];

    Instruction {
        program_id: ata_program,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(*wallet, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(*token_program, false),
        ],
        data,
    }
}
