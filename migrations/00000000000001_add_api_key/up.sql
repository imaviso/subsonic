-- Add API key column for OpenSubsonic apiKeyAuthentication extension
ALTER TABLE users ADD COLUMN api_key TEXT;

-- Index for API key lookups (should be unique but nullable)
CREATE UNIQUE INDEX idx_users_api_key ON users(api_key) WHERE api_key IS NOT NULL;
