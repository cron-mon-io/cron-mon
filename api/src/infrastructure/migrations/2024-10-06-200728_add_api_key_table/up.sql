CREATE TABLE api_key (
	api_key_id uuid PRIMARY KEY,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	key VARCHAR NOT NULL,
	tenant VARCHAR NOT NULL,
    last_used TIMESTAMP NULL,
	last_used_monitor_id uuid NULL,
    last_used_monitor_name VARCHAR NULL
);
SELECT diesel_manage_updated_at('api_key');
