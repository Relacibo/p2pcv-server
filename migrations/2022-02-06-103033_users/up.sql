CREATE EXTENSION pgcrypto;
CREATE TABLE users (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  name VARCHAR(20) NOT NULL,
  email VARCHAR(32) NOT NULL UNIQUE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('users');
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
CREATE POLICY users_access_account ON users USING (id = current_setting('jwt.claims.user')::uuid);
CREATE POLICY user_create_account ON users FOR
INSERT WITH CHECK (true);
CREATE VIEW users_view AS
SELECT id,
  name,
  created_at
FROM users;
CREATE INDEX users_email_index ON users (email);
CREATE TABLE user_sessions (
  id UUID PRIMARY KEY REFERENCES users,
  secret TEXT NOT NULL DEFAULT gen_salt('md5')
);