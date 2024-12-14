-- Add name to monitor_alert_config, nullable for now.
ALTER TABLE monitor_alert_config
    ADD monitor_name VARCHAR NULL;

-- Set the names to what they are currently, then make the column non-nullable.
UPDATE monitor_alert_config
    SET monitor_name = monitor.name
FROM monitor
WHERE monitor_alert_config.monitor_id = monitor.monitor_id;

ALTER TABLE monitor_alert_config
    ALTER COLUMN monitor_name SET NOT NULL;

-- Create a trigger function for updating the monitor name in monitor_alert_config.
CREATE OR REPLACE FUNCTION update_monitor_alert_config_monitor_name()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE monitor_alert_config
    SET monitor_name = NEW.name
    WHERE monitor_id = NEW.monitor_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply the trigger to the monitor table so that when name changes, the changes are
-- reflected in the monitor_alert_config table.
CREATE TRIGGER monitor_name_update
AFTER UPDATE OF name ON monitor
FOR EACH ROW
EXECUTE FUNCTION update_monitor_alert_config_monitor_name();
