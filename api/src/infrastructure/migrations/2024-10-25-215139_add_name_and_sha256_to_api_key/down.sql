-- Drop the indexes and constraints on the api_key table.
DROP INDEX idx_api_key_tenant;
DROP INDEX idx_api_key_key;
ALTER TABLE api_key
    DROP CONSTRAINT key_api_key_key;

-- Drop the name and masked columns.
ALTER TABLE api_key DROP name, DROP masked;
