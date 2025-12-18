use rust_decimal::Decimal;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::error::AppError;

pub struct SolanaClient {
    client: RpcClient,
    usdc_mint: Pubkey,
}

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub mint: String,
    pub amount: Decimal,
    pub decimals: u8,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, usdc_mint: &str) -> Self {
        let client = RpcClient::new(rpc_url.to_string());
        let usdc_mint = Pubkey::from_str(usdc_mint).expect("Invalid USDC mint address");

        Self { client, usdc_mint }
    }

    pub fn validate_address(address: &str) -> Result<Pubkey, AppError> {
        Pubkey::from_str(address)
            .map_err(|_| AppError::InvalidAddress(format!("Invalid Solana address: {}", address)))
    }

    pub fn get_usdc_balance(&self, wallet_address: &str) -> Result<TokenBalance, AppError> {
        let wallet_pubkey = Self::validate_address(wallet_address)?;

        // Get token accounts for the wallet
        let token_accounts = self
            .client
            .get_token_accounts_by_owner(
                &wallet_pubkey,
                solana_client::rpc_request::TokenAccountsFilter::Mint(self.usdc_mint),
            )
            .map_err(|e| AppError::SolanaRpc(e.to_string()))?;

        // Sum up all USDC balances (usually just one account)
        let mut total_amount: u64 = 0;
        let decimals: u8 = 6; // USDC has 6 decimals

        for account in token_accounts {
            if let solana_account_decoder::UiAccountData::Json(parsed) = account.account.data {
                if let Some(info) = parsed.parsed.get("info") {
                    if let Some(token_amount) = info.get("tokenAmount") {
                        if let Some(amount_str) = token_amount.get("amount") {
                            if let Some(amount) = amount_str.as_str() {
                                total_amount += amount.parse::<u64>().unwrap_or(0);
                            }
                        }
                    }
                }
            }
        }

        // Convert to decimal with proper decimals
        let amount = Decimal::new(total_amount as i64, decimals as u32);

        Ok(TokenBalance {
            mint: self.usdc_mint.to_string(),
            amount,
            decimals,
        })
    }

    pub fn get_signatures(
        &self,
        wallet_address: &str,
        limit: usize,
        before: Option<&str>,
    ) -> Result<Vec<String>, AppError> {
        let wallet_pubkey = Self::validate_address(wallet_address)?;

        let before_sig = before
            .map(|s| solana_sdk::signature::Signature::from_str(s))
            .transpose()
            .map_err(|_| AppError::InvalidAddress("Invalid signature".to_string()))?;

        let config = solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
            before: before_sig,
            until: None,
            limit: Some(limit),
            commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
        };

        let signatures = self
            .client
            .get_signatures_for_address_with_config(&wallet_pubkey, config)
            .map_err(|e| AppError::SolanaRpc(e.to_string()))?;

        Ok(signatures.into_iter().map(|s| s.signature).collect())
    }
}
