use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::ApyRateResponse;
use crate::error::AppError;
use crate::AppState;

// APY rates response
#[derive(Debug, Serialize)]
pub struct ApyRatesResponse {
    pub rates: Vec<ApyRateResponse>,
    pub count: usize,
}

pub async fn get_apy_rates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApyRatesResponse>, AppError> {
    let rates = state.apy.get_latest_rates().await?;
    let count = rates.len();

    Ok(Json(ApyRatesResponse { rates, count }))
}

// Best APY response
#[derive(Debug, Serialize)]
pub struct BestApyResponse {
    pub rate: Option<ApyRateResponse>,
}

pub async fn get_best_apy(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BestApyResponse>, AppError> {
    let rate = state.apy.get_best_rate().await?;

    Ok(Json(BestApyResponse { rate }))
}

// APY history query params
#[derive(Debug, Deserialize)]
pub struct ApyHistoryQuery {
    pub platform: String,
    pub hours: Option<i64>,
}

// APY history response
#[derive(Debug, Serialize)]
pub struct ApyHistoryResponse {
    pub platform: String,
    pub history: Vec<ApyRateResponse>,
    pub count: usize,
}

pub async fn get_apy_history(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ApyHistoryQuery>,
) -> Result<Json<ApyHistoryResponse>, AppError> {
    let hours = query.hours.unwrap_or(24);
    let history = state.apy.get_history(&query.platform, hours).await?;
    let count = history.len();

    Ok(Json(ApyHistoryResponse {
        platform: query.platform,
        history,
        count,
    }))
}

// Manual refresh APY rates
#[derive(Debug, Serialize)]
pub struct RefreshApyResponse {
    pub success: bool,
    pub rates_fetched: usize,
    pub message: String,
}

pub async fn refresh_apy_rates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RefreshApyResponse>, AppError> {
    match state.apy.fetch_and_store_rates().await {
        Ok(rates) => Ok(Json(RefreshApyResponse {
            success: true,
            rates_fetched: rates.len(),
            message: format!("Fetched {} APY rates from DeFiLlama", rates.len()),
        })),
        Err(e) => Ok(Json(RefreshApyResponse {
            success: false,
            rates_fetched: 0,
            message: format!("Failed to fetch rates: {}", e),
        })),
    }
}
