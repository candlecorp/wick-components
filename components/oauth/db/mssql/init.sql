IF NOT EXISTS (SELECT * FROM sys.objects WHERE object_id = OBJECT_ID(N'[dbo].[oauth_sessions]') AND type in (N'U'))
BEGIN
CREATE TABLE dbo.oauth_sessions (
  id NVARCHAR(50) PRIMARY KEY,
  token_type NVARCHAR(255) NOT NULL,
  access_token NVARCHAR(MAX) NOT NULL,
  refresh_token NVARCHAR(MAX) NOT NULL,
  expires_at DATETIME2 NOT NULL,
  created_at DATETIME2 NOT NULL DEFAULT GETDATE(),
  updated_at DATETIME2 NOT NULL DEFAULT GETDATE()
)
END

IF NOT EXISTS (SELECT * FROM sys.objects WHERE object_id = OBJECT_ID(N'[dbo].[oidc_claims]') AND type in (N'U'))
BEGIN
CREATE TABLE dbo.oidc_claims (
  session_id NVARCHAR(50) PRIMARY KEY FOREIGN KEY REFERENCES oauth_sessions(id),
  claims NVARCHAR(MAX) -- NVARCHAR(MAX) can hold JSON content, MSSQL does not have a JSON data type
)
END