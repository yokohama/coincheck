-- This file should undo anything in `up.sql`
ALTER TABLE orders DROP COLUMN spread_threshold;
ALTER TABLE orders DROP COLUMN api_call_success_at;
