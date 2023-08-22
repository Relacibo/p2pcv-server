-- This file should undo anything in `up.sql`
DELETE TABLE IF EXISTS lichess_users;
DELETE TABLE IF EXISTS lichess_access_tokens;