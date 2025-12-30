use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::AppState;

/// Build deposit transaction request
#[derive(Debug, Deserialize)]
pub struct BuildDepositRequest {
    pub wallet: String,
    pub amount: f64,
    pub protocol: String, // "kamino" or "save"
}

/// Build deposit transaction response
#[derive(Debug, Serialize)]
pub struct BuildDepositResponse {
    pub transaction: String,
    pub blockhash: String,
    pub last_valid_block_height: u64,
    pub protocol: String,
    pub amount_lamports: u64,
}

pub async fn build_deposit_transaction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildDepositRequest>,
) -> Result<Json<BuildDepositResponse>, AppError> {
    // Validate amount
    if req.amount <= 0.0 {
        return Err(AppError::BadRequest("Amount must be positive".into()));
    }

    match req.protocol.as_str() {
        "kamino" => {
            let result = state.deposit.build_kamino_deposit(&req.wallet, req.amount).await?;
            Ok(Json(BuildDepositResponse {
                transaction: result.transaction,
                blockhash: result.blockhash,
                last_valid_block_height: result.last_valid_block_height,
                protocol: result.protocol,
                amount_lamports: result.amount_lamports,
            }))
        }
        "save" => {
            // Save/Solend not implemented yet
            Err(AppError::BadRequest("Save protocol deposits not yet implemented".into()))
        }
        _ => Err(AppError::BadRequest(format!("Unknown protocol: {}", req.protocol))),
    }
}
