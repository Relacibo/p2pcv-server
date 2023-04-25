-- Your SQL goes here
CREATE TABLE friend_requests (
  id BIGSERIAL PRIMARY KEY,
  sender_id UUID NOT NULL REFERENCES users(id),
  receiver_id UUID NOT NULL REFERENCES users(id),
  message VARCHAR NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (sender_id, receiver_id),
  CHECK (sender_id != receiver_id)
);