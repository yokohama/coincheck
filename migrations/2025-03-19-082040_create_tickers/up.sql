CREATE TABLE tickers (
    id SERIAL PRIMARY KEY,
    pair TEXT NOT NULL,
    last FLOAT8 NOT NULL,
    bid FLOAT8 NOT NULL,
    ask FLOAT8 NOT NULL,
    high FLOAT8 NOT NULL,
    low FLOAT8 NOT NULL,
    volume FLOAT8 NOT NULL,
    timestamp TIMESTAMP DEFAULT NOW()
);

