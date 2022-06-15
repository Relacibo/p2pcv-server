CREATE EXTENSION pgcrypto;
CREATE TABLE users (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
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
CREATE TABLE users_google (
  id UUID PRIMARY KEY REFERENCES users,
  google_id VARCHAR(255) NOT NULL UNIQUE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX users_google_id_index ON users_google (google_id);