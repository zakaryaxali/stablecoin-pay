//! Shared RPC helper methods for deposit operations.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use super::constants::{CONFIRMATION_MAX_RETRIES, CONFIRMATION_RETRY_DELAY_MS, PUBKEY_SIZE};
use super::types::{AccountResult, BlockhashResult, StatusResult};
use super::DepositService;
use crate::error::AppError;
use crate::services::rpc_types::RpcResponse;

impl DepositService {
    /// Fetch raw account data from RPC
    pub(super) async fn fetch_account_data(&self, address: &Pubkey) -> Result<Vec<u8>, AppError> {
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

    /// Detect which token program owns a mint account
    /// Returns TOKEN_PROGRAM or TOKEN_2022_PROGRAM pubkey
    pub(super) async fn detect_token_program(&self, mint: &Pubkey) -> Result<Pubkey, AppError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [mint.to_string(), {"encoding": "base64"}]
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
            .ok_or_else(|| AppError::SolanaRpc("Mint account not found".to_string()))?;

        // The owner field tells us which token program owns this mint
        Pubkey::from_str(&account_data.owner)
            .map_err(|_| AppError::SolanaRpc("Invalid owner pubkey".to_string()))
    }

    /// Parse a Pubkey from raw bytes at a given offset
    pub(super) fn parse_pubkey(&self, data: &[u8], offset: usize) -> Result<Pubkey, AppError> {
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
    pub(super) async fn get_recent_blockhash(&self) -> Result<(String, u64), AppError> {
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
    /// Polls getSignatureStatuses until confirmed or timeout.
    /// Note: Once a transaction is accepted (we have a signature), it will be processed
    /// regardless of block height. The block height only matters for initial submission.
    pub async fn confirm_transaction(
        &self,
        signature: &str,
        _blockhash: &str,
        _last_valid_block_height: u64,
    ) -> Result<bool, AppError> {
        let max_retries = CONFIRMATION_MAX_RETRIES;
        let retry_delay = std::time::Duration::from_millis(CONFIRMATION_RETRY_DELAY_MS);

        for _ in 0..max_retries {
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

            tokio::time::sleep(retry_delay).await;
        }

        Err(AppError::SolanaRpc(
            "Transaction confirmation timeout".to_string(),
        ))
    }
}
