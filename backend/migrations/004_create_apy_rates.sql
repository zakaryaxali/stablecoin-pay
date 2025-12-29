-- APY rates from DeFi protocols
CREATE TABLE IF NOT EXISTS apy_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(50) NOT NULL,
    chain VARCHAR(50) NOT NULL DEFAULT 'solana',
    token VARCHAR(20) NOT NULL DEFAULT 'USDC',
    apy_total DECIMAL(10, 4) NOT NULL,
    apy_base DECIMAL(10, 4),
    apy_reward DECIMAL(10, 4),
    tvl_usd DECIMAL(20, 2),
    pool_id VARCHAR(100),
    source VARCHAR(50) NOT NULL DEFAULT 'defillama',
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for querying latest rates by platform
CREATE INDEX idx_apy_rates_platform_fetched ON apy_rates(platform, fetched_at DESC);

-- Index for querying by chain and token
CREATE INDEX idx_apy_rates_chain_token ON apy_rates(chain, token);
