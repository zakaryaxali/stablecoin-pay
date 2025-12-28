use chrono::{DateTime, TimeZone, Utc};
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
    pub usdc_mint: String,
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

// Transaction response types for getTransaction
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransactionResult {
    block_time: Option<i64>,
    meta: Option<TransactionMeta>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransactionMeta {
    pre_token_balances: Option<Vec<TokenBalanceMeta>>,
    post_token_balances: Option<Vec<TokenBalanceMeta>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenBalanceMeta {
    owner: Option<String>,
    mint: Option<String>,
    ui_token_amount: Option<UiTokenAmount>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UiTokenAmount {
    ui_amount: Option<f64>,
    amount: String,
    decimals: u8,
}

/// Parsed transaction ready for database storage
#[derive(Debug, Clone)]
pub struct ParsedTransaction {
    pub signature: String,
    pub wallet_address: String,
    pub tx_type: String, // "send" or "receive"
    pub amount: Decimal,
    pub token_mint: String,
    pub counterparty: String,
    pub block_time: DateTime<Utc>,
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

    /// Fetch and parse a single transaction to extract USDC transfer details
    pub async fn get_transaction_details(
        &self,
        signature: &str,
        wallet_address: &str,
    ) -> Result<Option<ParsedTransaction>, AppError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                signature,
                {
                    "encoding": "jsonParsed",
                    "maxSupportedTransactionVersion": 0
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

        let rpc_response: RpcResponse<TransactionResult> = response
            .json()
            .await
            .map_err(|e| AppError::SolanaRpc(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = rpc_response.error {
            return Err(AppError::SolanaRpc(error.message));
        }

        let result = match rpc_response.result {
            Some(r) => r,
            None => return Ok(None), // Transaction not found
        };

        let block_time = result
            .block_time
            .map(|ts| Utc.timestamp_opt(ts, 0).single())
            .flatten()
            .unwrap_or_else(Utc::now);

        // Get token balance metadata
        let meta = match result.meta {
            Some(m) => m,
            None => return Ok(None),
        };

        let pre_balances = meta.pre_token_balances.unwrap_or_default();
        let post_balances = meta.post_token_balances.unwrap_or_default();

        // Find USDC balances for our wallet in pre and post
        let mut our_pre_balance: Option<u64> = None;
        let mut our_post_balance: Option<u64> = None;
        let mut counterparty: Option<String> = None;

        // Check pre-balances for our wallet's USDC
        for balance in &pre_balances {
            if balance.owner.as_deref() == Some(wallet_address)
                && balance.mint.as_deref() == Some(&self.usdc_mint)
            {
                if let Some(ref ui_amount) = balance.ui_token_amount {
                    our_pre_balance = ui_amount.amount.parse().ok();
                }
            }
        }

        // Check post-balances for our wallet's USDC
        for balance in &post_balances {
            if balance.owner.as_deref() == Some(wallet_address)
                && balance.mint.as_deref() == Some(&self.usdc_mint)
            {
                if let Some(ref ui_amount) = balance.ui_token_amount {
                    our_post_balance = ui_amount.amount.parse().ok();
                }
            }
        }

        // Find counterparty (other wallet involved in USDC transfer)
        for balance in post_balances.iter().chain(pre_balances.iter()) {
            if balance.mint.as_deref() == Some(&self.usdc_mint)
                && balance.owner.as_deref() != Some(wallet_address)
            {
                if let Some(owner) = &balance.owner {
                    counterparty = Some(owner.clone());
                    break;
                }
            }
        }

        // Determine transaction type based on balance change
        let (tx_type, amount_raw) = match (our_pre_balance, our_post_balance) {
            (Some(pre), Some(post)) if post > pre => ("receive", post - pre),
            (Some(pre), Some(post)) if pre > post => ("send", pre - post),
            (None, Some(post)) if post > 0 => ("receive", post), // New account with balance
            (Some(pre), None) if pre > 0 => ("send", pre),       // Account closed
            _ => return Ok(None), // No change or not related to this wallet
        };

        if amount_raw == 0 {
            return Ok(None);
        }

        let amount = Decimal::new(amount_raw as i64, 6); // USDC has 6 decimals

        Ok(Some(ParsedTransaction {
            signature: signature.to_string(),
            wallet_address: wallet_address.to_string(),
            tx_type: tx_type.to_string(),
            amount,
            token_mint: self.usdc_mint.clone(),
            counterparty: counterparty.unwrap_or_else(|| "unknown".to_string()),
            block_time,
        }))
    }

    /// Sync recent transactions for a wallet from the blockchain
    pub async fn sync_wallet_transactions(
        &self,
        wallet_address: &str,
        limit: usize,
    ) -> Result<Vec<ParsedTransaction>, AppError> {
        // Get recent signatures
        let signatures = self.get_signatures(wallet_address, limit, None).await?;

        let mut transactions = Vec::new();

        // Fetch details for each signature
        for signature in signatures {
            match self
                .get_transaction_details(&signature, wallet_address)
                .await
            {
                Ok(Some(tx)) => transactions.push(tx),
                Ok(None) => {} // Not a USDC transfer, skip
                Err(e) => {
                    // Log error but continue with other transactions
                    tracing::warn!("Failed to fetch transaction {}: {}", signature, e);
                }
            }
        }

        Ok(transactions)
    }
}
