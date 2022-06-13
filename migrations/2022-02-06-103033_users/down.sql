DROP INDEX session_cookie_index;
DROP TABLE user_sessions;
DROP VIEW users_view;
DROP POLICY users_access_account ON users;
DROP TABLE users;
DROP EXTENSION pgcrypto;