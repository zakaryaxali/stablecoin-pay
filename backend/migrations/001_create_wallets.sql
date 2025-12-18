-- Create wallets table
CREATE TABLE IF NOT EXISTS wallets (
    address VARCHAR(44) PRIMARY KEY,
    webhook_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
