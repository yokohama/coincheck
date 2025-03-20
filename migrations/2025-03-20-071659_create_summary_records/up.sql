CREATE TABLE summary_records (
    id SERIAL PRIMARY KEY,
    summary_id INTEGER NOT NULL,
    currency VARCHAR(255) NOT NULL,
    amount FLOAT8 NOT NULL,
    rate FLOAT8 NOT NULL,
    jpy_value FLOAT8 NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
