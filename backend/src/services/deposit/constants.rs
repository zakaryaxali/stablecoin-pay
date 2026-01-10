//! Constants for deposit/withdraw operations across lending protocols.

// ─────────────────────────────────────────────────────────────────────────────
// Common Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Associated Token Program
pub const ASSOCIATED_TOKEN_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// System Sysvar Instructions
pub const SYSVAR_INSTRUCTIONS: &str = "Sysvar1nstructions1111111111111111111111111";

/// System Program
pub const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";

/// SPL Token Program
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// SPL Token-2022 Program (unused but kept for reference)
#[allow(dead_code)]
pub const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

/// USDC token configuration (6 decimals)
pub const USDC_DECIMAL_MULTIPLIER: f64 = 1_000_000.0;

/// Solana pubkey size in bytes
pub const PUBKEY_SIZE: usize = 32;

/// Transaction confirmation settings
pub const CONFIRMATION_MAX_RETRIES: u32 = 30;
pub const CONFIRMATION_RETRY_DELAY_MS: u64 = 500;

// ─────────────────────────────────────────────────────────────────────────────
// Kamino Protocol Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Kamino Lend program ID
pub const KAMINO_PROGRAM_ID: &str = "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD";

/// Kamino Main Market
pub const KAMINO_LENDING_MARKET: &str = "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF";

/// Kamino USDC Reserve (mainnet)
pub const USDC_RESERVE: &str = "D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59";

// Kamino Reserve account layout offsets (after 8-byte Anchor discriminator)
pub const RESERVE_LIQUIDITY_MINT_OFFSET: usize = 128;
pub const RESERVE_LIQUIDITY_SUPPLY_OFFSET: usize = 160;
pub const RESERVE_LIQUIDITY_TOKEN_PROGRAM_OFFSET: usize = 408;
pub const RESERVE_COLLATERAL_MINT_OFFSET: usize = 2560;
pub const RESERVE_COLLATERAL_SUPPLY_OFFSET: usize = 2600;

// ─────────────────────────────────────────────────────────────────────────────
// Save (Solend) Protocol Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Save (Solend) program ID
pub const SAVE_PROGRAM_ID: &str = "So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo";

/// Save Main Pool Lending Market
pub const SAVE_LENDING_MARKET: &str = "4UpD2fh7xH3VP9QQaXtsS1YY3bxzWhtfpks7FatyKvdY";

/// Save USDC Reserve (Main Pool)
pub const SAVE_USDC_RESERVE: &str = "BgxfHJDzm44T7XG68MYKx7YisTjZu73tVovyZSjJMpmw";

// Save Reserve account layout offsets (no Anchor discriminator)
pub const SAVE_RESERVE_LIQUIDITY_MINT_OFFSET: usize = 42;
pub const SAVE_RESERVE_LIQUIDITY_SUPPLY_OFFSET: usize = 75;
pub const SAVE_RESERVE_COLLATERAL_MINT_OFFSET: usize = 227;
#[allow(dead_code)]
pub const SAVE_RESERVE_COLLATERAL_SUPPLY_OFFSET: usize = 259;

// Save instruction codes (non-Anchor format)
pub const SAVE_DEPOSIT_INSTRUCTION_CODE: u8 = 4;
pub const SAVE_WITHDRAW_INSTRUCTION_CODE: u8 = 5;
