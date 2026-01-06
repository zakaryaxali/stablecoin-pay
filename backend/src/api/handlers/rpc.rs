use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::AppState;

/// Send transaction request
#[derive(Debug, Deserialize)]
pub struct SendTransactionRequest {
    pub transaction: String, // base64 encoded signed transaction
}

/// Send transaction response
#[derive(Debug, Serialize)]
pub struct SendTransactionResponse {
    pub signature: String,
}

/// Confirm transaction request
#[derive(Debug, Deserialize)]
pub struct ConfirmTransactionRequest {
    pub signature: String,
    pub blockhash: String,
    pub last_valid_block_height: u64,
}

/// Confirm transaction response
#[derive(Debug, Serialize)]
pub struct ConfirmTransactionResponse {
    pub confirmed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Send a signed transaction to the Solana network
pub async fn send_transaction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SendTransactionRequest>,
) -> Result<Json<SendTransactionResponse>, AppError> {
    let signature = state.deposit.send_raw_transaction(&req.transaction).await?;
    Ok(Json(SendTransactionResponse { signature }))
}

/// Confirm a transaction on the Solana network
pub async fn confirm_transaction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConfirmTransactionRequest>,
) -> Result<Json<ConfirmTransactionResponse>, AppError> {
    match state
        .deposit
        .confirm_transaction(&req.signature, &req.blockhash, req.last_valid_block_height)
        .await
    {
        Ok(confirmed) => Ok(Json(ConfirmTransactionResponse {
            confirmed,
            error: None,
        })),
        Err(e) => Ok(Json(ConfirmTransactionResponse {
            confirmed: false,
            error: Some(e.to_string()),
        })),
    }
}
