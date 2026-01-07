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
use sha2::{Digest, Sha256};
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    transaction::Transaction,
};
use std::str::FromStr;

use crate::error::AppError;
use crate::services::rpc_types::RpcResponse;

// ─────────────────────────────────────────────────────────────────────────────
// RPC Response Types
// ─────────────────────────────────────────────────────────────────────────────

/// Account info response value
#[derive(Deserialize)]
struct AccountValue {
    data: (String, String), // (base64_data, encoding)
}

/// Account info result wrapper
#[derive(Deserialize)]
struct AccountResult {
    value: Option<AccountValue>,
}

/// Blockhash response value
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlockhashValue {
    blockhash: String,
    last_valid_block_height: u64,
}

/// Blockhash result wrapper
#[derive(Deserialize)]
struct BlockhashResult {
    value: BlockhashValue,
}

/// Signature status for transaction confirmation
#[derive(Deserialize, Debug)]
struct SignatureStatus {
    #[allow(dead_code)]
    slot: Option<u64>,
    #[allow(dead_code)]
    confirmations: Option<u64>,
    err: Option<serde_json::Value>,
    #[serde(rename = "confirmationStatus")]
    confirmation_status: Option<String>,
}

/// Signature status result wrapper
#[derive(Deserialize)]
struct StatusResult {
    value: Vec<Option<SignatureStatus>>,
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

/// Derive Associated Token Account address (same as spl_associated_token_account::get_associated_token_address)
fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    let ata_program = Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM).unwrap();

    let (ata, _) = Pubkey::find_program_address(
        &[wallet.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ata_program,
    );
    ata
}

/// Build create_associated_token_account_idempotent instruction
/// Creates an ATA if it doesn't exist, does nothing if it already exists
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
            AccountMeta::new(*payer, true),            // payer (signer, writable)
            AccountMeta::new(ata, false),              // associated token account (writable)
            AccountMeta::new_readonly(*wallet, false), // wallet address
            AccountMeta::new_readonly(*mint, false),   // token mint
            AccountMeta::new_readonly(system_program, false), // system program
            AccountMeta::new_readonly(*token_program, false), // token program
        ],
        data,
    }
}

/// Kamino Lend program ID
const KAMINO_PROGRAM_ID: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

/// Kamino Main Market
const KAMINO_LENDING_MARKET: &str = "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF";

/// Kamino USDC Reserve (mainnet) - hardcoded known address
const USDC_RESERVE: &str = "D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59";

/// Associated Token Program
const ASSOCIATED_TOKEN_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// System Sysvar Instructions
const SYSVAR_INSTRUCTIONS: &str = "Sysvar1nstructions1111111111111111111111111";

/// System Program
const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";

// Kamino Reserve account layout offsets (after 8-byte Anchor discriminator)
const RESERVE_LIQUIDITY_MINT_OFFSET: usize = 128;
const RESERVE_LIQUIDITY_SUPPLY_OFFSET: usize = 160;
const RESERVE_LIQUIDITY_TOKEN_PROGRAM_OFFSET: usize = 408;
const RESERVE_COLLATERAL_MINT_OFFSET: usize = 2560;
const RESERVE_COLLATERAL_SUPPLY_OFFSET: usize = 2600;

// USDC token configuration
const USDC_DECIMAL_MULTIPLIER: f64 = 1_000_000.0; // 10^6

// Solana data sizes
const PUBKEY_SIZE: usize = 32;

// Transaction confirmation settings
const CONFIRMATION_MAX_RETRIES: u32 = 30;
const CONFIRMATION_RETRY_DELAY_MS: u64 = 500;

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

        // Convert USDC amount to lamports (6 decimals)
        let amount_lamports = (amount_usdc * USDC_DECIMAL_MULTIPLIER) as u64;

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
        let user_source_liquidity = get_associated_token_address(
            &owner,
            &reserve.liquidity_mint,
            &reserve.liquidity_token_program,
        );

        // User's destination collateral token account (kToken ATA)
        let user_destination_collateral = get_associated_token_address(
            &owner,
            &reserve.collateral_mint,
            &reserve.collateral_token_program,
        );

        // Build instruction to create collateral ATA if it doesn't exist
        let create_ata_instruction = build_create_ata_idempotent_instruction(
            &owner,
            &owner,
            &reserve.collateral_mint,
            &reserve.collateral_token_program,
        );

        // Build the deposit instruction
        let deposit_instruction = self.build_deposit_instruction(
            &owner,
            &reserve,
            &lending_market,
            &lending_market_authority,
            &user_source_liquidity,
            &user_destination_collateral,
            amount_lamports,
        )?;

        // Create the transaction with blockhash (create ATA first, then deposit)
        let blockhash_hash = Hash::from_str(&blockhash)
            .map_err(|_| AppError::SolanaRpc("Invalid blockhash".to_string()))?;
        let message = Message::new_with_blockhash(
            &[create_ata_instruction, deposit_instruction],
            Some(&owner),
            &blockhash_hash,
        );

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
            AccountMeta::new(*owner, true),                    // owner (signer)
            AccountMeta::new(reserve.reserve_address, false),  // reserve
            AccountMeta::new_readonly(*lending_market, false), // lending_market
            AccountMeta::new_readonly(*lending_market_authority, false), // lending_market_authority
            AccountMeta::new_readonly(reserve.liquidity_mint, false), // reserve_liquidity_mint
            AccountMeta::new(reserve.liquidity_supply, false), // reserve_liquidity_supply
            AccountMeta::new(reserve.collateral_mint, false),  // reserve_collateral_mint
            AccountMeta::new(*user_source_liquidity, false),   // user_source_liquidity
            AccountMeta::new(*user_destination_collateral, false), // user_destination_collateral
            AccountMeta::new_readonly(reserve.collateral_token_program, false), // collateral_token_program
            AccountMeta::new_readonly(reserve.liquidity_token_program, false), // liquidity_token_program
            AccountMeta::new_readonly(sysvar_instructions, false), // instruction_sysvar_account
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
    ///
    /// Parses the on-chain Reserve account to extract real addresses instead of
    /// deriving PDAs with guessed seeds (which caused transaction reverts).
    async fn get_kamino_usdc_reserve(&self) -> Result<KaminoReserveData, AppError> {
        let reserve_address = Pubkey::from_str(USDC_RESERVE).unwrap();

        // Fetch the reserve account data from RPC
        let reserve_data = self.fetch_account_data(&reserve_address).await?;

        // Parse reserve account data using known offsets from Kamino's Reserve struct
        let liquidity_mint = self.parse_pubkey(&reserve_data, RESERVE_LIQUIDITY_MINT_OFFSET)?;
        let liquidity_supply = self.parse_pubkey(&reserve_data, RESERVE_LIQUIDITY_SUPPLY_OFFSET)?;
        let liquidity_token_program =
            self.parse_pubkey(&reserve_data, RESERVE_LIQUIDITY_TOKEN_PROGRAM_OFFSET)?;
        let collateral_mint = self.parse_pubkey(&reserve_data, RESERVE_COLLATERAL_MINT_OFFSET)?;
        let collateral_supply =
            self.parse_pubkey(&reserve_data, RESERVE_COLLATERAL_SUPPLY_OFFSET)?;

        // Collateral token program is same as liquidity for USDC
        let collateral_token_program = liquidity_token_program;

        Ok(KaminoReserveData {
            reserve_address,
            liquidity_mint,
            liquidity_supply,
            collateral_mint,
            collateral_supply,
            liquidity_token_program,
            collateral_token_program,
        })
    }

    /// Get the Kamino collateral (kToken) mint address for USDC
    ///
    /// This is the token users receive when depositing USDC to Kamino.
    pub async fn get_kamino_collateral_mint(&self) -> Result<String, AppError> {
        let reserve = self.get_kamino_usdc_reserve().await?;
        Ok(reserve.collateral_mint.to_string())
    }

    /// Fetch raw account data from RPC
    async fn fetch_account_data(&self, address: &Pubkey) -> Result<Vec<u8>, AppError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [address.to_string(), {"encoding": "base64"}]
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Request failed: {}", e)))?;

        let rpc_response: RpcResponse<AccountResult> = response
            .json()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AppError::SolanaRpc(format!("RPC error: {:?}", error)));
        }

        let account_data = rpc_response
            .result
            .and_then(|r| r.value)
            .ok_or_else(|| AppError::SolanaRpc("Account not found".to_string()))?;

        STANDARD
            .decode(&account_data.data.0)
            .map_err(|e| AppError::SolanaRpc(format!("Failed to decode base64: {}", e)))
    }

    /// Parse a Pubkey from raw bytes at a given offset
    fn parse_pubkey(&self, data: &[u8], offset: usize) -> Result<Pubkey, AppError> {
        if offset + PUBKEY_SIZE > data.len() {
            return Err(AppError::SolanaRpc(format!(
                "Offset {} out of bounds for data length {}",
                offset,
                data.len()
            )));
        }

        let bytes: [u8; PUBKEY_SIZE] = data[offset..offset + PUBKEY_SIZE]
            .try_into()
            .map_err(|_| AppError::SolanaRpc("Failed to parse pubkey bytes".to_string()))?;

        Ok(Pubkey::from(bytes))
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

        let rpc_response: RpcResponse<BlockhashResult> = response
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

    /// Send a signed transaction to the Solana network
    ///
    /// This proxies the sendTransaction RPC call to avoid browser CORS/rate-limit issues.
    pub async fn send_raw_transaction(&self, tx_base64: &str) -> Result<String, AppError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [
                tx_base64,
                {
                    "encoding": "base64",
                    "skipPreflight": false,
                    "preflightCommitment": "confirmed"
                }
            ]
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Request failed: {}", e)))?;

        let rpc_response: RpcResponse<String> = response
            .json()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AppError::SolanaRpc(format!(
                "Transaction failed: {:?}",
                error
            )));
        }

        rpc_response
            .result
            .ok_or_else(|| AppError::SolanaRpc("No signature in response".to_string()))
    }

    /// Confirm a transaction on the Solana network
    ///
    /// Polls getSignatureStatuses until confirmed or block height exceeded.
    pub async fn confirm_transaction(
        &self,
        signature: &str,
        _blockhash: &str,
        last_valid_block_height: u64,
    ) -> Result<bool, AppError> {
        // Poll for confirmation with timeout
        let max_retries = CONFIRMATION_MAX_RETRIES;
        let retry_delay = std::time::Duration::from_millis(CONFIRMATION_RETRY_DELAY_MS);

        for _ in 0..max_retries {
            // Check signature status
            let body = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getSignatureStatuses",
                "params": [[signature]]
            });

            let response = self
                .client
                .post(&self.rpc_url)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::SolanaRpc(format!("Request failed: {}", e)))?;

            let rpc_response: RpcResponse<StatusResult> = response
                .json()
                .await
                .map_err(|e| AppError::SolanaRpc(format!("Failed to parse response: {}", e)))?;

            if let Some(error) = rpc_response.error {
                return Err(AppError::SolanaRpc(format!("RPC error: {:?}", error)));
            }

            if let Some(result) = rpc_response.result {
                if let Some(Some(status)) = result.value.into_iter().next() {
                    // Check for transaction error
                    if status.err.is_some() {
                        return Err(AppError::SolanaRpc(format!(
                            "Transaction error: {:?}",
                            status.err
                        )));
                    }

                    // Check confirmation status
                    if let Some(conf_status) = status.confirmation_status {
                        if conf_status == "confirmed" || conf_status == "finalized" {
                            return Ok(true);
                        }
                    }
                }
            }

            // Check if block height exceeded
            let (_, current_height) = self.get_recent_blockhash().await?;
            if current_height > last_valid_block_height {
                return Err(AppError::SolanaRpc(
                    "Transaction expired: block height exceeded".to_string(),
                ));
            }

            tokio::time::sleep(retry_delay).await;
        }

        Err(AppError::SolanaRpc(
            "Transaction confirmation timeout".to_string(),
        ))
    }
}
