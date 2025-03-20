CREATE TABLE summaries (
    id SERIAL PRIMARY KEY,
    total_invested FLOAT8 NOT NULL,
    total_jpy_value FLOAT8 NOT NULL,
    pl FLOAT8 NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
