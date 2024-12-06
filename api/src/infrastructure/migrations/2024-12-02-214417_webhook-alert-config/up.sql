CREATE TABLE webhook_alert_config (
    alert_config_id uuid PRIMARY KEY REFERENCES alert_config ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- These columns are intentionally overly verbose in how they are named,
    -- since we're using class table inheritance, and we fetch all alert configs
    -- in one query using left joins. If we weren't as verbose here, the columns
    -- could class (for example if we add support for Discord alerts, would
    -- 'channel' be for Discord or for Slack?
    webhook_url VARCHAR NOT NULL
);

SELECT diesel_manage_updated_at('webhook_alert_config');
