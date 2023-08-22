-- Your SQL goes here
CREATE TABLE lichess_users (
  id VARCHAR NOT NULL PRIMARY KEY,
  username VARCHAR NOT NULL,
  user_id UUID NOT NULL UNIQUE REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT
  diesel_manage_updated_at('lichess_users');