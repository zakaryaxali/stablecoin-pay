use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Send,
    Receive,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Send => write!(f, "send"),
            TransactionType::Receive => write!(f, "receive"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Confirmed,
    Pending,
    Failed,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Confirmed => write!(f, "confirmed"),
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub signature: String,
    pub wallet_address: String,
    pub tx_type: TransactionType,
    pub amount: Decimal,
    pub token_mint: String,
    pub counterparty: String,
    pub status: TransactionStatus,
    pub block_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
