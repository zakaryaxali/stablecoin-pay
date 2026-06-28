//! Kamino protocol deposit/withdraw implementation.

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
    KAMINO_LENDING_MARKET, KAMINO_PROGRAM_ID, RESERVE_COLLATERAL_MINT_OFFSET,
    RESERVE_COLLATERAL_SUPPLY_OFFSET, RESERVE_LIQUIDITY_MINT_OFFSET,
    RESERVE_LIQUIDITY_SUPPLY_OFFSET, RESERVE_LIQUIDITY_TOKEN_PROGRAM_OFFSET, SYSVAR_INSTRUCTIONS,
    USDC_DECIMAL_MULTIPLIER, USDC_RESERVE,
};
use super::types::{BuildDepositResponse, KaminoReserveData};
use super::{
    anchor_discriminator, build_create_ata_idempotent_instruction, get_associated_token_address,
    DepositService,
};
use crate::error::AppError;

impl DepositService {
    /// Build an unsigned Kamino deposit transaction
    pub async fn build_kamino_deposit(
        &self,
        wallet_address: &str,
        amount_usdc: f64,
    ) -> Result<BuildDepositResponse, AppError> {
        let owner = Pubkey::from_str(wallet_address)
            .map_err(|_| AppError::InvalidAddress(wallet_address.to_string()))?;

        let amount_lamports = (amount_usdc * USDC_DECIMAL_MULTIPLIER) as u64;

        let reserve = self.get_kamino_usdc_reserve().await?;
        let (blockhash, last_valid_block_height) = self.get_recent_blockhash().await?;

        let lending_market = Pubkey::from_str(KAMINO_LENDING_MARKET).unwrap();
        let program_id = Pubkey::from_str(KAMINO_PROGRAM_ID).unwrap();

        // Lending market authority PDA
        let (lending_market_authority, _) =
            Pubkey::find_program_address(&[b"lma", lending_market.as_ref()], &program_id);

        // User's ATAs
        let user_source_liquidity = get_associated_token_address(
            &owner,
            &reserve.liquidity_mint,
            &reserve.liquidity_token_program,
        );
        let user_destination_collateral = get_associated_token_address(
            &owner,
            &reserve.collateral_mint,
            &reserve.collateral_token_program,
        );

        // Build instructions
        let create_ata_instruction = build_create_ata_idempotent_instruction(
            &owner,
            &owner,
            &reserve.collateral_mint,
            &reserve.collateral_token_program,
        );

        let deposit_instruction = self.build_kamino_deposit_instruction(
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
            protocol: "kamino".to_string(),
            amount_lamports,
        })
    }

    /// Build an unsigned Kamino withdraw (redeem) transaction
    pub async fn build_kamino_withdraw(
        &self,
        wallet_address: &str,
        amount_ktokens: f64,
    ) -> Result<BuildDepositResponse, AppError> {
        let owner = Pubkey::from_str(wallet_address)
            .map_err(|_| AppError::InvalidAddress(wallet_address.to_string()))?;

        let amount_lamports = (amount_ktokens * USDC_DECIMAL_MULTIPLIER) as u64;

        let reserve = self.get_kamino_usdc_reserve().await?;
        let (blockhash, last_valid_block_height) = self.get_recent_blockhash().await?;

        let lending_market = Pubkey::from_str(KAMINO_LENDING_MARKET).unwrap();
        let program_id = Pubkey::from_str(KAMINO_PROGRAM_ID).unwrap();

        let (lending_market_authority, _) =
            Pubkey::find_program_address(&[b"lma", lending_market.as_ref()], &program_id);

        let user_source_collateral = get_associated_token_address(
            &owner,
            &reserve.collateral_mint,
            &reserve.collateral_token_program,
        );
        let user_destination_liquidity = get_associated_token_address(
            &owner,
            &reserve.liquidity_mint,
            &reserve.liquidity_token_program,
        );

        let withdraw_instruction = self.build_kamino_withdraw_instruction(
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
            protocol: "kamino".to_string(),
            amount_lamports,
        })
    }

    /// Get the Kamino collateral (kToken) mint address for USDC
    pub async fn get_kamino_collateral_mint(&self) -> Result<String, AppError> {
        let reserve = self.get_kamino_usdc_reserve().await?;
        Ok(reserve.collateral_mint.to_string())
    }

    /// Fetch Kamino USDC reserve data from chain
    async fn get_kamino_usdc_reserve(&self) -> Result<KaminoReserveData, AppError> {
        let reserve_address = Pubkey::from_str(USDC_RESERVE).unwrap();
        let reserve_data = self.fetch_account_data(&reserve_address).await?;

        let liquidity_mint = self.parse_pubkey(&reserve_data, RESERVE_LIQUIDITY_MINT_OFFSET)?;
        let liquidity_supply = self.parse_pubkey(&reserve_data, RESERVE_LIQUIDITY_SUPPLY_OFFSET)?;
        let liquidity_token_program =
            self.parse_pubkey(&reserve_data, RESERVE_LIQUIDITY_TOKEN_PROGRAM_OFFSET)?;
        let collateral_mint = self.parse_pubkey(&reserve_data, RESERVE_COLLATERAL_MINT_OFFSET)?;
        let collateral_supply =
            self.parse_pubkey(&reserve_data, RESERVE_COLLATERAL_SUPPLY_OFFSET)?;
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

    /// Build the deposit_reserve_liquidity instruction
    #[allow(clippy::too_many_arguments)]
    fn build_kamino_deposit_instruction(
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

        let accounts = vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(reserve.reserve_address, false),
            AccountMeta::new_readonly(*lending_market, false),
            AccountMeta::new_readonly(*lending_market_authority, false),
            AccountMeta::new_readonly(reserve.liquidity_mint, false),
            AccountMeta::new(reserve.liquidity_supply, false),
            AccountMeta::new(reserve.collateral_mint, false),
            AccountMeta::new(*user_source_liquidity, false),
            AccountMeta::new(*user_destination_collateral, false),
            AccountMeta::new_readonly(reserve.collateral_token_program, false),
            AccountMeta::new_readonly(reserve.liquidity_token_program, false),
            AccountMeta::new_readonly(sysvar_instructions, false),
        ];

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

    /// Build the redeem_reserve_collateral instruction (withdraw/redeem)
    #[allow(clippy::too_many_arguments)]
    fn build_kamino_withdraw_instruction(
        &self,
        owner: &Pubkey,
        reserve: &KaminoReserveData,
        lending_market: &Pubkey,
        lending_market_authority: &Pubkey,
        user_source_collateral: &Pubkey,
        user_destination_liquidity: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, AppError> {
        let program_id = Pubkey::from_str(KAMINO_PROGRAM_ID).unwrap();
        let sysvar_instructions = Pubkey::from_str(SYSVAR_INSTRUCTIONS).unwrap();

        let accounts = vec![
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new_readonly(*lending_market, false),
            AccountMeta::new(reserve.reserve_address, false),
            AccountMeta::new_readonly(*lending_market_authority, false),
            AccountMeta::new_readonly(reserve.liquidity_mint, false),
            AccountMeta::new(reserve.collateral_mint, false),
            AccountMeta::new(reserve.liquidity_supply, false),
            AccountMeta::new(*user_source_collateral, false),
            AccountMeta::new(*user_destination_liquidity, false),
            AccountMeta::new_readonly(reserve.collateral_token_program, false),
            AccountMeta::new_readonly(reserve.liquidity_token_program, false),
            AccountMeta::new_readonly(sysvar_instructions, false),
        ];

        let discriminator = anchor_discriminator("redeem_reserve_collateral");
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&discriminator);
        data.extend_from_slice(&amount.to_le_bytes());

        Ok(Instruction {
            program_id,
            accounts,
            data,
        })
    }
}
