-- Add protocol field for identifying staking protocol (kamino, save, or null for regular transfers)
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS protocol VARCHAR(20);

-- Add index for filtering by protocol
CREATE INDEX IF NOT EXISTS idx_transactions_protocol ON transactions(protocol);
