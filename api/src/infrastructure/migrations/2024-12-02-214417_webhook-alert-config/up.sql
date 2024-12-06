CREATE TABLE webhook_alert_config (
    alert_config_id uuid PRIMARY KEY REFERENCES alert_config ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    url VARCHAR NOT NULL
);

SELECT diesel_manage_updated_at('webhook_alert_config');
