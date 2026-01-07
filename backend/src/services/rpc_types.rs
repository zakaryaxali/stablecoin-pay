//! Shared types for Solana JSON-RPC responses.

use serde::Deserialize;

/// Generic JSON-RPC response wrapper for Solana RPC calls.
///
/// All Solana RPC methods return this structure with either a result or an error.
#[derive(Debug, Deserialize)]
pub struct RpcResponse<T> {
    pub result: Option<T>,
    pub error: Option<serde_json::Value>,
}
