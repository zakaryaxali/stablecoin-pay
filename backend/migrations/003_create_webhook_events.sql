-- Webhook events table for tracking webhook delivery attempts
CREATE TABLE IF NOT EXISTS webhook_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(44) NOT NULL REFERENCES wallets(address) ON DELETE CASCADE,
    transaction_signature VARCHAR(88) REFERENCES transactions(signature) ON DELETE SET NULL,
    event_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'delivered', 'failed')),
    attempts INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for finding pending webhooks to retry
CREATE INDEX idx_webhook_events_status ON webhook_events(status) WHERE status = 'pending';

-- Index for finding webhooks by wallet
CREATE INDEX idx_webhook_events_wallet ON webhook_events(wallet_address, created_at DESC);

-- Index for finding webhooks by transaction
CREATE INDEX idx_webhook_events_transaction ON webhook_events(transaction_signature);
