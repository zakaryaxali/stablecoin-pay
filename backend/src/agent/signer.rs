//! Treasury key custody and cluster safety for the agent.
//!
//! The agent is the one part of this codebase that signs autonomously, so two
//! safety rails live here:
//!
//! 1. [`ensure_safe_cluster`] refuses to run against a non-devnet RPC unless
//!    `AGENT_ALLOW_MAINNET=true` is set explicitly. The default config prefers
//!    a Helius mainnet endpoint, so the agent path must opt in to a devnet RPC.
//! 2. [`load_treasury_keypair`] loads the treasury secret from a base58 string
//!    or a Solana CLI keypair file. **The secret is never logged.**

use std::path::Path;

use solana_sdk::signature::Keypair;

use crate::error::AppError;

/// Reject a non-devnet RPC unless mainnet has been explicitly allowed.
///
/// A URL is considered safe when `allow_mainnet` is set, or when the URL names
/// the devnet cluster. Anything else (mainnet, testnet, localhost, an opaque
/// custom endpoint) is rejected so the agent cannot move real funds by default.
pub fn ensure_safe_cluster(rpc_url: &str, allow_mainnet: bool) -> Result<(), AppError> {
    if allow_mainnet {
        return Ok(());
    }

    if rpc_url.to_lowercase().contains("devnet") {
        return Ok(());
    }

    Err(AppError::Internal(format!(
        "agent refuses to run against non-devnet RPC '{rpc_url}'; \
         set SOLANA_RPC_URL to a devnet endpoint or AGENT_ALLOW_MAINNET=true to override"
    )))
}

/// Load the treasury [`Keypair`] from a base58 secret key or a keypair file.
///
/// If `source` is a path to an existing file it is read as a Solana CLI keypair
/// (a JSON array of 64 bytes); otherwise it is treated as a base58-encoded
/// 64-byte secret key. The secret is never logged.
pub fn load_treasury_keypair(source: &str) -> Result<Keypair, AppError> {
    let bytes = if Path::new(source).is_file() {
        let contents = std::fs::read_to_string(source)
            .map_err(|e| AppError::Internal(format!("failed to read agent keypair file: {e}")))?;
        serde_json::from_str::<Vec<u8>>(&contents).map_err(|e| {
            AppError::Internal(format!("agent keypair file is not a byte array: {e}"))
        })?
    } else {
        bs58::decode(source.trim())
            .into_vec()
            .map_err(|e| AppError::Internal(format!("agent keypair is not valid base58: {e}")))?
    };

    Keypair::try_from(&bytes[..])
        .map_err(|e| AppError::Internal(format!("agent keypair bytes are invalid: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Signer;

    #[test]
    fn loads_a_base58_keypair() {
        let original = Keypair::new();
        let secret_b58 = bs58::encode(original.to_bytes()).into_string();

        let loaded = load_treasury_keypair(&secret_b58).expect("keypair should load");

        assert_eq!(loaded.pubkey(), original.pubkey());
    }

    #[test]
    fn rejects_garbage_keypair() {
        assert!(load_treasury_keypair("not-a-real-key").is_err());
    }

    #[test]
    fn mainnet_guard_rejects_mainnet_rpc() {
        let result = ensure_safe_cluster("https://api.mainnet-beta.solana.com", false);
        assert!(result.is_err(), "mainnet RPC must be rejected by default");
    }

    #[test]
    fn mainnet_guard_rejects_helius_mainnet() {
        let result = ensure_safe_cluster("https://mainnet.helius-rpc.com/?api-key=secret", false);
        assert!(
            result.is_err(),
            "helius mainnet RPC must be rejected by default"
        );
    }

    #[test]
    fn devnet_rpc_is_allowed() {
        assert!(ensure_safe_cluster("https://api.devnet.solana.com", false).is_ok());
    }

    #[test]
    fn mainnet_allowed_when_overridden() {
        assert!(ensure_safe_cluster("https://api.mainnet-beta.solana.com", true).is_ok());
    }
}
