CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_name VARCHAR NOT NULL UNIQUE,
  name VARCHAR NOT NULL,
  nick_name VARCHAR NULL,
  given_name VARCHAR NULL,
  middle_name VARCHAR NULL,
  family_name VARCHAR NULL,
  email VARCHAR NOT NULL UNIQUE,
  locale VARCHAR NOT NULL DEFAULT 'en',
  verified_email BOOLEAN NOT NULL DEFAULT FALSE,
  picture VARCHAR NULL, 
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

CREATE TABLE peers (
  peer_id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

