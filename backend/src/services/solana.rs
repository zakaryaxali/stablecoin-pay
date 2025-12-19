use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::error::AppError;

pub struct SolanaClient {
    client: Client,
    rpc_url: String,
    usdc_mint: String,
}

#[derive(Debug, Clone)]
pub struct TokenBalance {
    pub mint: String,
    pub amount: Decimal,
    pub decimals: u8,
}

// JSON-RPC response types
#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct TokenAccountsResult {
    value: Vec<TokenAccountInfo>,
}

#[derive(Debug, Deserialize)]
struct TokenAccountInfo {
    account: AccountData,
}

#[derive(Debug, Deserialize)]
struct AccountData {
    data: ParsedData,
}

#[derive(Debug, Deserialize)]
struct ParsedData {
    parsed: ParsedInfo,
}

#[derive(Debug, Deserialize)]
struct ParsedInfo {
    info: TokenInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenInfo {
    token_amount: TokenAmount,
}

#[derive(Debug, Deserialize)]
struct TokenAmount {
    amount: String,
    decimals: u8,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, usdc_mint: &str) -> Self {
        let client = Client::new();

        Self {
            client,
            rpc_url: rpc_url.to_string(),
            usdc_mint: usdc_mint.to_string(),
        }
    }

    pub fn validate_address(address: &str) -> Result<Pubkey, AppError> {
        Pubkey::from_str(address)
            .map_err(|_| AppError::InvalidAddress(format!("Invalid Solana address: {}", address)))
    }

    pub async fn get_usdc_balance(&self, wallet_address: &str) -> Result<TokenBalance, AppError> {
        // Validate address
        Self::validate_address(wallet_address)?;

        // Build JSON-RPC request for getTokenAccountsByOwner
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTokenAccountsByOwner",
            "params": [
                wallet_address,
                { "mint": self.usdc_mint },
                { "encoding": "jsonParsed" }
            ]
        });

        // Make the request
        let response = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Request failed: {}", e)))?;

        let rpc_response: RpcResponse<TokenAccountsResult> = response
            .json()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Failed to parse response: {}", e)))?;

        // Check for RPC error
        if let Some(error) = rpc_response.error {
            return Err(AppError::SolanaRpc(error.message));
        }

        // Extract balance from response
        let result = rpc_response
            .result
            .ok_or_else(|| AppError::SolanaRpc("No result in response".to_string()))?;

        let mut total_amount: u64 = 0;
        let decimals: u8 = 6; // USDC has 6 decimals

        for account in result.value {
            let amount_str = &account.account.data.parsed.info.token_amount.amount;
            total_amount += amount_str.parse::<u64>().unwrap_or(0);
        }

        // Convert to decimal with proper decimals
        let amount = Decimal::new(total_amount as i64, decimals as u32);

        Ok(TokenBalance {
            mint: self.usdc_mint.clone(),
            amount,
            decimals,
        })
    }

    pub async fn get_signatures(
        &self,
        wallet_address: &str,
        limit: usize,
        _before: Option<&str>,
    ) -> Result<Vec<String>, AppError> {
        // Validate address
        Self::validate_address(wallet_address)?;

        // Build JSON-RPC request for getSignaturesForAddress
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignaturesForAddress",
            "params": [
                wallet_address,
                { "limit": limit }
            ]
        });

        // Make the request
        let response = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Request failed: {}", e)))?;

        #[derive(Debug, Deserialize)]
        struct SignatureInfo {
            signature: String,
        }

        let rpc_response: RpcResponse<Vec<SignatureInfo>> = response
            .json()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AppError::SolanaRpc(error.message));
        }

        let result = rpc_response
            .result
            .ok_or_else(|| AppError::SolanaRpc("No result in response".to_string()))?;

        Ok(result.into_iter().map(|s| s.signature).collect())
    }
}
