-- Remove API key column
DROP INDEX IF EXISTS idx_users_api_key;
ALTER TABLE users DROP COLUMN api_key;
