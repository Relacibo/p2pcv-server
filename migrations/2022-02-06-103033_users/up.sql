CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR NULL,
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