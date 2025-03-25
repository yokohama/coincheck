ALTER TABLE orders RENAME COLUMN crypto_amount TO amount;
ALTER TABLE orders DROP COLUMN jpy_amount;
