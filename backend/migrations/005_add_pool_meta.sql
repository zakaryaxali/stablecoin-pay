-- Add pool_meta column for human-readable pool names from DeFiLlama
ALTER TABLE apy_rates ADD COLUMN IF NOT EXISTS pool_meta VARCHAR(100);
