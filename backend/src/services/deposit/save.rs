//! Save (Solend) protocol deposit/withdraw implementation.

use base64::{engine::general_purpose::STANDARD, Engine};
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    transaction::Transaction,
};
use std::str::FromStr;

use super::constants::{
    SAVE_DEPOSIT_INSTRUCTION_CODE, SAVE_LENDING_MARKET, SAVE_PROGRAM_ID,
    SAVE_RESERVE_COLLATERAL_MINT_OFFSET, SAVE_RESERVE_COLLATERAL_SUPPLY_OFFSET,
    SAVE_RESERVE_LIQUIDITY_MINT_OFFSET, SAVE_RESERVE_LIQUIDITY_SUPPLY_OFFSET, SAVE_USDC_RESERVE,
    SAVE_WITHDRAW_INSTRUCTION_CODE, TOKEN_PROGRAM, USDC_DECIMAL_MULTIPLIER,
};
use super::types::{BuildDepositResponse, SaveReserveData};
use super::{
    build_create_ata_idempotent_instruction, get_associated_token_address, DepositService,
};
use crate::error::AppError;

impl DepositService {
    /// Build an unsigned Save deposit transaction
    pub async fn build_save_deposit(
        &self,
        wallet_address: &str,
        amount_usdc: f64,
    ) -> Result<BuildDepositResponse, AppError> {
        let owner = Pubkey::from_str(wallet_address)
            .map_err(|_| AppError::InvalidAddress(wallet_address.to_string()))?;

        let amount_lamports = (amount_usdc * USDC_DECIMAL_MULTIPLIER) as u64;

        let reserve = self.get_save_usdc_reserve().await?;
        let (blockhash, last_valid_block_height) = self.get_recent_blockhash().await?;

        let lending_market = Pubkey::from_str(SAVE_LENDING_MARKET).unwrap();
        let program_id = Pubkey::from_str(SAVE_PROGRAM_ID).unwrap();
        let token_program = Pubkey::from_str(TOKEN_PROGRAM).unwrap();

        tracing::debug!(
            "Save deposit: collateral_mint={}, detected_token_program={}",
            reserve.collateral_mint,
            reserve.collateral_token_program
        );

        // Lending market authority PDA
        let (lending_market_authority, _) =
            Pubkey::find_program_address(&[lending_market.as_ref()], &program_id);

        // User's ATAs (both use standard token program)
        let user_source_liquidity =
            get_associated_token_address(&owner, &reserve.liquidity_mint, &token_program);
        let user_destination_collateral =
            get_associated_token_address(&owner, &reserve.collateral_mint, &token_program);

        // Build instructions
        let create_ata_instruction = build_create_ata_idempotent_instruction(
            &owner,
            &owner,
            &reserve.collateral_mint,
            &token_program,
        );

        let deposit_instruction = self.build_save_deposit_instruction(
            &owner,
            &reserve,
            &lending_market,
            &lending_market_authority,
            &user_source_liquidity,
            &user_destination_collateral,
            amount_lamports,
        )?;

        // Create transaction
        let blockhash_hash = Hash::from_str(&blockhash)
            .map_err(|_| AppError::SolanaRpc("Invalid blockhash".to_string()))?;
        let message = Message::new_with_blockhash(
            &[create_ata_instruction, deposit_instruction],
            Some(&owner),
            &blockhash_hash,
        );

        let transaction = Transaction::new_unsigned(message);
        let tx_bytes = bincode::serialize(&transaction)
            .map_err(|e| AppError::SolanaRpc(format!("Failed to serialize transaction: {}", e)))?;
        let tx_base64 = STANDARD.encode(&tx_bytes);

        Ok(BuildDepositResponse {
            transaction: tx_base64,
            blockhash,
            last_valid_block_height,
            protocol: "save".to_string(),
            amount_lamports,
        })
    }

    /// Build an unsigned Save withdraw (redeem) transaction
    pub async fn build_save_withdraw(
        &self,
        wallet_address: &str,
        amount_ctokens: f64,
    ) -> Result<BuildDepositResponse, AppError> {
        let owner = Pubkey::from_str(wallet_address)
            .map_err(|_| AppError::InvalidAddress(wallet_address.to_string()))?;

        let amount_lamports = (amount_ctokens * USDC_DECIMAL_MULTIPLIER) as u64;

        let reserve = self.get_save_usdc_reserve().await?;
        let (blockhash, last_valid_block_height) = self.get_recent_blockhash().await?;

        let lending_market = Pubkey::from_str(SAVE_LENDING_MARKET).unwrap();
        let program_id = Pubkey::from_str(SAVE_PROGRAM_ID).unwrap();
        let token_program = Pubkey::from_str(TOKEN_PROGRAM).unwrap();

        let (lending_market_authority, _) =
            Pubkey::find_program_address(&[lending_market.as_ref()], &program_id);

        let user_source_collateral =
            get_associated_token_address(&owner, &reserve.collateral_mint, &token_program);
        let user_destination_liquidity =
            get_associated_token_address(&owner, &reserve.liquidity_mint, &token_program);

        let withdraw_instruction = self.build_save_withdraw_instruction(
            &owner,
            &reserve,
            &lending_market,
            &lending_market_authority,
            &user_source_collateral,
            &user_destination_liquidity,
            amount_lamports,
        )?;

        let blockhash_hash = Hash::from_str(&blockhash)
            .map_err(|_| AppError::SolanaRpc("Invalid blockhash".to_string()))?;
        let message =
            Message::new_with_blockhash(&[withdraw_instruction], Some(&owner), &blockhash_hash);

        let transaction = Transaction::new_unsigned(message);
        let tx_bytes = bincode::serialize(&transaction)
            .map_err(|e| AppError::SolanaRpc(format!("Failed to serialize transaction: {}", e)))?;
        let tx_base64 = STANDARD.encode(&tx_bytes);

        Ok(BuildDepositResponse {
            transaction: tx_base64,
            blockhash,
            last_valid_block_height,
            protocol: "save".to_string(),
            amount_lamports,
        })
    }

    /// Get the Save collateral (cToken) mint address for USDC
    pub async fn get_save_collateral_mint(&self) -> Result<String, AppError> {
        let reserve = self.get_save_usdc_reserve().await?;
        Ok(reserve.collateral_mint.to_string())
    }

    /// Fetch Save USDC reserve data from chain
    async fn get_save_usdc_reserve(&self) -> Result<SaveReserveData, AppError> {
        let reserve_address = Pubkey::from_str(SAVE_USDC_RESERVE).unwrap();
        let reserve_data = self.fetch_account_data(&reserve_address).await?;

        let liquidity_mint =
            self.parse_pubkey(&reserve_data, SAVE_RESERVE_LIQUIDITY_MINT_OFFSET)?;
        let liquidity_supply =
            self.parse_pubkey(&reserve_data, SAVE_RESERVE_LIQUIDITY_SUPPLY_OFFSET)?;
        let collateral_mint =
            self.parse_pubkey(&reserve_data, SAVE_RESERVE_COLLATERAL_MINT_OFFSET)?;
        let collateral_supply =
            self.parse_pubkey(&reserve_data, SAVE_RESERVE_COLLATERAL_SUPPLY_OFFSET)?;

        let collateral_token_program = self.detect_token_program(&collateral_mint).await?;

        Ok(SaveReserveData {
            reserve_address,
            liquidity_mint,
            liquidity_supply,
            collateral_mint,
            collateral_supply,
            collateral_token_program,
        })
    }

    /// Build the Save deposit_reserve_liquidity instruction
    #[allow(clippy::too_many_arguments)]
    fn build_save_deposit_instruction(
        &self,
        owner: &Pubkey,
        reserve: &SaveReserveData,
        lending_market: &Pubkey,
        lending_market_authority: &Pubkey,
        user_source_liquidity: &Pubkey,
        user_destination_collateral: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, AppError> {
        let program_id = Pubkey::from_str(SAVE_PROGRAM_ID).unwrap();
        let token_program = Pubkey::from_str(TOKEN_PROGRAM).unwrap();

        // 9 accounts total (clock sysvar removed in newer versions)
        let accounts = vec![
            AccountMeta::new(*user_source_liquidity, false),
            AccountMeta::new(*user_destination_collateral, false),
            AccountMeta::new(reserve.reserve_address, false),
            AccountMeta::new(reserve.liquidity_supply, false),
            AccountMeta::new(reserve.collateral_mint, false),
            AccountMeta::new_readonly(*lending_market, false),
            AccountMeta::new_readonly(*lending_market_authority, false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new_readonly(token_program, false),
        ];

        let mut data = Vec::with_capacity(9);
        data.push(SAVE_DEPOSIT_INSTRUCTION_CODE);
        data.extend_from_slice(&amount.to_le_bytes());

        Ok(Instruction {
            program_id,
            accounts,
            data,
        })
    }

    /// Build the Save redeem_reserve_collateral instruction (withdraw)
    #[allow(clippy::too_many_arguments)]
    fn build_save_withdraw_instruction(
        &self,
        owner: &Pubkey,
        reserve: &SaveReserveData,
        lending_market: &Pubkey,
        lending_market_authority: &Pubkey,
        user_source_collateral: &Pubkey,
        user_destination_liquidity: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, AppError> {
        let program_id = Pubkey::from_str(SAVE_PROGRAM_ID).unwrap();
        let token_program = Pubkey::from_str(TOKEN_PROGRAM).unwrap();

        // 9 accounts total (clock sysvar removed in newer versions)
        let accounts = vec![
            AccountMeta::new(*user_source_collateral, false),
            AccountMeta::new(*user_destination_liquidity, false),
            AccountMeta::new(reserve.reserve_address, false),
            AccountMeta::new(reserve.collateral_mint, false),
            AccountMeta::new(reserve.liquidity_supply, false),
            AccountMeta::new(*lending_market, false),
            AccountMeta::new_readonly(*lending_market_authority, false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new_readonly(token_program, false),
        ];

        let mut data = Vec::with_capacity(9);
        data.push(SAVE_WITHDRAW_INSTRUCTION_CODE);
        data.extend_from_slice(&amount.to_le_bytes());

        Ok(Instruction {
            program_id,
            accounts,
            data,
        })
    }
}
