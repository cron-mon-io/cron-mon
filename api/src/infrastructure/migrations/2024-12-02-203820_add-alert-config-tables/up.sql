CREATE TABLE alert_config (
	alert_config_id uuid PRIMARY KEY,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

	name VARCHAR NOT NULL,
	tenant VARCHAR NOT NULL,
	type VARCHAR NOT NULL,
	active BOOLEAN NOT NULL,
    on_late BOOLEAN NOT NULL,
    on_error BOOLEAN NOT NULL
);

CREATE TABLE slack_alert_config (
    alert_config_id uuid PRIMARY KEY REFERENCES alert_config ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- These columns are intentionally overly verbose in how they are named,
    -- since we're using class table inheritance, and we fetch all alert configs
    -- in one query using left joins. If we weren't as verbose here, the columns
    -- could class (for example if we add support for Discord alerts, would
    -- 'channel' be for Discord or for Slack?
    slack_channel VARCHAR NOT NULL,
    slack_bot_oauth_token VARCHAR NOT NULL
);

-- This is an association table between alert_config and monitor.
CREATE TABLE monitor_alert_config (
	alert_config_id uuid REFERENCES alert_config ON DELETE CASCADE,
    monitor_id uuid REFERENCES monitor ON DELETE CASCADE,

    CONSTRAINT pk_monitor_alert_config PRIMARY KEY (alert_config_id, monitor_id)
);

SELECT diesel_manage_updated_at('alert_config');
SELECT diesel_manage_updated_at('slack_alert_config');
