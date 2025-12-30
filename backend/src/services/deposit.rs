//! Deposit transaction builder for Kamino (and future Save) protocols.
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

use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Sha256, Digest};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    transaction::Transaction,
};
use std::str::FromStr;

use crate::error::AppError;

/// Compute Anchor instruction discriminator: sha256("global:<name>")[0..8]
fn anchor_discriminator(name: &str) -> [u8; 8] {
    let preimage = format!("global:{}", name);
    let hash = Sha256::digest(preimage.as_bytes());
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash[..8]);
    discriminator
}

/// Derive Associated Token Account address (same as spl_associated_token_account::get_associated_token_address)
fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    let token_program = Pubkey::from_str(TOKEN_PROGRAM).unwrap();
    let ata_program = Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM).unwrap();

    let (ata, _) = Pubkey::find_program_address(
        &[
            wallet.as_ref(),
            token_program.as_ref(),
            mint.as_ref(),
        ],
        &ata_program,
    );
    ata
}

/// Kamino Lend program ID
const KAMINO_PROGRAM_ID: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

/// Kamino Main Market
const KAMINO_LENDING_MARKET: &str = "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF";

/// USDC Mint
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

/// Token Program
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// Associated Token Program
const ASSOCIATED_TOKEN_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// System Sysvar Instructions
const SYSVAR_INSTRUCTIONS: &str = "Sysvar1nstructions1111111111111111111111111";

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
    pub collateral_supply: Pubkey,
    pub liquidity_token_program: Pubkey,
    pub collateral_token_program: Pubkey,
}

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

    /// Build an unsigned Kamino deposit transaction
    pub async fn build_kamino_deposit(
        &self,
        wallet_address: &str,
        amount_usdc: f64,
    ) -> Result<BuildDepositResponse, AppError> {
        let owner = Pubkey::from_str(wallet_address)
            .map_err(|_| AppError::InvalidAddress(wallet_address.to_string()))?;

        // Convert USDC amount to micro usdc (~ 10**3 lamports) (6 decimals)
        let amount_lamports = (amount_usdc * 1_000_000.0) as u64;

        // Fetch reserve data for USDC on Kamino
        let reserve = self.get_kamino_usdc_reserve().await?;

        // Get recent blockhash and last valid block height
        let (blockhash, last_valid_block_height) = self.get_recent_blockhash().await?;

        // Derive PDAs and associated token accounts
        let lending_market = Pubkey::from_str(KAMINO_LENDING_MARKET).unwrap();
        let program_id = Pubkey::from_str(KAMINO_PROGRAM_ID).unwrap();

        // Lending market authority PDA
        let (lending_market_authority, _) =
            Pubkey::find_program_address(&[b"lma", lending_market.as_ref()], &program_id);

        // User's source USDC token account (ATA)
        let user_source_liquidity = get_associated_token_address(&owner, &reserve.liquidity_mint);

        // User's destination collateral token account (kToken ATA)
        let user_destination_collateral = get_associated_token_address(&owner, &reserve.collateral_mint);

        // Build the deposit instruction
        let instruction = self.build_deposit_instruction(
            &owner,
            &reserve,
            &lending_market,
            &lending_market_authority,
            &user_source_liquidity,
            &user_destination_collateral,
            amount_lamports,
        )?;

        // Create the transaction
        let message = Message::new(&[instruction], Some(&owner));

        let transaction = Transaction::new_unsigned(message);

        // Serialize to base64
        let tx_bytes = bincode::serialize(&transaction)
            .map_err(|e| AppError::SolanaRpc(format!("Failed to serialize transaction: {}", e)))?;
        let tx_base64 = STANDARD.encode(&tx_bytes);

        Ok(BuildDepositResponse {
            transaction: tx_base64,
            blockhash,
            last_valid_block_height,
            protocol: "kamino".to_string(),
            amount_lamports,
        })
    }

    /// Build the deposit_reserve_liquidity instruction
    fn build_deposit_instruction(
        &self,
        owner: &Pubkey,
        reserve: &KaminoReserveData,
        lending_market: &Pubkey,
        lending_market_authority: &Pubkey,
        user_source_liquidity: &Pubkey,
        user_destination_collateral: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, AppError> {
        let program_id = Pubkey::from_str(KAMINO_PROGRAM_ID).unwrap();
        let sysvar_instructions = Pubkey::from_str(SYSVAR_INSTRUCTIONS).unwrap();

        // Account metas for DepositReserveLiquidity instruction
        // Order matters! Must match the Kamino program's expected order
        let accounts = vec![
            AccountMeta::new(*owner, true),                          // owner (signer)
            AccountMeta::new(reserve.reserve_address, false),        // reserve
            AccountMeta::new_readonly(*lending_market, false),       // lending_market
            AccountMeta::new_readonly(*lending_market_authority, false), // lending_market_authority
            AccountMeta::new_readonly(reserve.liquidity_mint, false), // reserve_liquidity_mint
            AccountMeta::new(reserve.liquidity_supply, false),       // reserve_liquidity_supply
            AccountMeta::new(reserve.collateral_mint, false),        // reserve_collateral_mint
            AccountMeta::new(*user_source_liquidity, false),         // user_source_liquidity
            AccountMeta::new(*user_destination_collateral, false),   // user_destination_collateral
            AccountMeta::new_readonly(reserve.collateral_token_program, false), // collateral_token_program
            AccountMeta::new_readonly(reserve.liquidity_token_program, false),  // liquidity_token_program
            AccountMeta::new_readonly(sysvar_instructions, false),   // instruction_sysvar_account
        ];

        // Instruction data: discriminator + amount (u64 LE)
        let discriminator = anchor_discriminator("deposit_reserve_liquidity");
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&discriminator);
        data.extend_from_slice(&amount.to_le_bytes());

        Ok(Instruction {
            program_id,
            accounts,
            data,
        })
    }

    /// Fetch Kamino USDC reserve data from chain
    async fn get_kamino_usdc_reserve(&self) -> Result<KaminoReserveData, AppError> {
        // Known Kamino USDC reserve address on mainnet
        // This is derived from: PDA([lending_market, usdc_mint], kamino_program)
        let lending_market = Pubkey::from_str(KAMINO_LENDING_MARKET).unwrap();
        let usdc_mint = Pubkey::from_str(USDC_MINT).unwrap();
        let program_id = Pubkey::from_str(KAMINO_PROGRAM_ID).unwrap();

        // Find reserve PDA
        let (reserve_address, _) = Pubkey::find_program_address(
            &[
                b"reserve",
                lending_market.as_ref(),
                usdc_mint.as_ref(),
            ],
            &program_id,
        );

        // Parse reserve data to extract relevant fields
        // The Reserve struct is complex, so we derive related accounts using known PDA patterns

        // Collateral mint PDA
        let (collateral_mint, _) = Pubkey::find_program_address(
            &[
                b"reserve_coll_mint",
                reserve_address.as_ref(),
            ],
            &program_id,
        );

        // Liquidity supply PDA
        let (liquidity_supply, _) = Pubkey::find_program_address(
            &[
                b"reserve_liq_supply",
                reserve_address.as_ref(),
            ],
            &program_id,
        );

        // Collateral supply PDA
        let (collateral_supply, _) = Pubkey::find_program_address(
            &[
                b"reserve_coll_supply",
                reserve_address.as_ref(),
            ],
            &program_id,
        );

        Ok(KaminoReserveData {
            reserve_address,
            liquidity_mint: usdc_mint,
            liquidity_supply,
            collateral_mint,
            collateral_supply,
            liquidity_token_program: Pubkey::from_str(TOKEN_PROGRAM).unwrap(),
            collateral_token_program: Pubkey::from_str(TOKEN_PROGRAM).unwrap(),
        })
    }

    /// Get recent blockhash and last valid block height from RPC
    async fn get_recent_blockhash(&self) -> Result<(String, u64), AppError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLatestBlockhash",
            "params": []
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Request failed: {}", e)))?;

        #[derive(Deserialize)]
        struct BlockhashResult {
            value: BlockhashValue,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct BlockhashValue {
            blockhash: String,
            last_valid_block_height: u64,
        }

        #[derive(Deserialize)]
        struct RpcResponse {
            result: Option<BlockhashResult>,
            error: Option<serde_json::Value>,
        }

        let rpc_response: RpcResponse = response
            .json()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AppError::SolanaRpc(format!("RPC error: {:?}", error)));
        }

        rpc_response
            .result
            .map(|r| (r.value.blockhash, r.value.last_valid_block_height))
            .ok_or_else(|| AppError::SolanaRpc("No blockhash in response".to_string()))
    }
}
