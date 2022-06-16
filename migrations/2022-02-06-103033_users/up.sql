CREATE EXTENSION pgcrypto;
CREATE SCHEMA auth;
CREATE TABLE auth.user (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  name VARCHAR(20) NOT NULL,
  email VARCHAR(32) NOT NULL UNIQUE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('auth.user');
ALTER TABLE auth.user ENABLE ROW LEVEL SECURITY;
CREATE POLICY users_access_account ON auth.user USING (id = current_setting('jwt.claims.user')::uuid);
CREATE VIEW auth.user_view AS
SELECT id,
  name,
  created_at
FROM auth.user;