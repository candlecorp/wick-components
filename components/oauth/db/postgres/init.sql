CREATE TABLE IF NOT EXISTS oauth_sessions (
  id VARCHAR(50) PRIMARY KEY,
  token_type VARCHAR(255) NOT NULL,
  access_token TEXT NOT NULL,
  id_token TEXT NOT NULL,
  refresh_token TEXT NOT NULL,
  expires_at TIMESTAMP NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS oidc_claims (
  session_id VARCHAR(50) PRIMARY KEY REFERENCES oauth_sessions(id),
  claims json
);