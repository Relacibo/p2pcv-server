CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_name VARCHAR NOT NULL UNIQUE,
  display_name VARCHAR NOT NULL,
  email VARCHAR NOT NULL UNIQUE,
  locale VARCHAR NOT NULL DEFAULT 'en',
  verified_email BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT
  diesel_manage_updated_at('users');

CREATE TABLE friends (
  id BIGSERIAL PRIMARY KEY,
  user1_id UUID NOT NULL REFERENCES users(id),
  user2_id UUID NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (user1_id, user2_id),
  CHECK (user2_id > user1_id)
);
