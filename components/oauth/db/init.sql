CREATE TABLE IF NOT EXISTS oauth_sessions (
  id SERIAL PRIMARY KEY,
  access_token TEXT NOT NULL,
  refresh_token TEXT NOT NULL,
  expires_at TIMESTAMP NOT NULL
  session_id VARCHAR(255)
);