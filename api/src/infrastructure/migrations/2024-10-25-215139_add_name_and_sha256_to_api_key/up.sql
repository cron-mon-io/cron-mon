ALTER TABLE api_key
	ADD name VARCHAR NULL,
    ADD masked VARCHAR NULL;

-- Add a name and masked version of the key to each record.
UPDATE api_key
SET name = 'API Key ' || SUBSTRING(key FROM 1 FOR 5),
    masked = SUBSTRING(key FROM 1 FOR 5) || '************' || SUBSTRING(key FROM LENGTH(key) - 5 FOR 6);


-- The key can be sha256 hashed with the following query, but we can't do it in the migration
-- because we'd need a means of reversing it, and sha256 is irreversinble. This query is here
-- for reference.
-- UPDATE api_key
-- SET key = encode(digest(key, 'sha256'), 'hex');

ALTER TABLE api_key
    ALTER COLUMN name SET NOT NULL,
    ALTER COLUMN masked SET NOT NULL;

-- Add missing index on the tenant and key column.
CREATE INDEX idx_api_key_tenant ON api_key(tenant);
CREATE INDEX idx_api_key_key ON api_key(key);

-- Add a unique constraint on the key column.
ALTER TABLE api_key
    ADD CONSTRAINT key_api_key_key UNIQUE (key);
