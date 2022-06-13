CREATE EXTENSION pgcrypto;
CREATE TABLE users (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  name VARCHAR(20) NOT NULL,
  email VARCHAR(32) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(email)
);
SELECT diesel_manage_updated_at('users');
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
CREATE POLICY users_access_account ON users USING (true);
CREATE VIEW users_view AS
SELECT id,
  name,
  created_at
FROM users;
CREATE TABLE user_sessions (
  id UUID PRIMARY KEY REFERENCES users,
  secret TEXT NOT NULL DEFAULT gen_salt('md5')
);
CREATE INDEX session_cookie_index ON user_sessions(
  (
    id || '-' || encode(hmac(id::text, secret, 'sha256'), 'hex')
  )
);