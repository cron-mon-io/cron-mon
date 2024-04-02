CREATE TABLE monitor (
	monitor_id uuid PRIMARY KEY,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	name VARCHAR NOT NULL,
	expected_duration INT NOT NULL,
	grace_duration INT NOT NULL
);
SELECT diesel_manage_updated_at('monitor');
