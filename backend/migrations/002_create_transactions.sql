-- Create transactions table
CREATE TABLE IF NOT EXISTS transactions (
    signature VARCHAR(88) PRIMARY KEY,
    wallet_address VARCHAR(44) NOT NULL REFERENCES wallets(address) ON DELETE CASCADE,
    tx_type VARCHAR(10) NOT NULL CHECK (tx_type IN ('send', 'receive')),
    amount DECIMAL(20, 6) NOT NULL,
    token_mint VARCHAR(44) NOT NULL,
    counterparty VARCHAR(44) NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('confirmed', 'pending', 'failed')),
    block_time TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_transactions_wallet ON transactions(wallet_address);
CREATE INDEX IF NOT EXISTS idx_transactions_block_time ON transactions(block_time DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_wallet_time ON transactions(wallet_address, block_time DESC);
