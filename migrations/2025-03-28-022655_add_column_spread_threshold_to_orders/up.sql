ALTER TABLE orders ADD COLUMN spread_threshold FLOAT8;
ALTER TABLE orders ADD COLUMN api_call_success_at TIMESTAMP DEFAULT NOW();
