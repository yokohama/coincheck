ALTER TABLE orders RENAME COLUMN amount TO crypto_amount;
ALTER TABLE orders ADD COLUMN jpy_amount FLOAT8;
