CREATE TABLE optimized_mas (
  id SERIAL PRIMARY KEY,
  pair TEXT NOT NULL,
  short_ma INT NOT NULL,
  long_ma INT NOT NULL,
  offset_minutes INT NOT NULL,
  win_rate_pct FLOAT8,
  total INT,
  wins INT,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

