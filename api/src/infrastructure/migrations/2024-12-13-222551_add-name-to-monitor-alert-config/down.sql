-- Drop trigger and function
DROP TRIGGER IF EXISTS monitor_name_update ON monitor;
DROP FUNCTION IF EXISTS update_monitor_alert_config_monitor_name;

ALTER TABLE monitor_alert_config DROP monitor_name;
