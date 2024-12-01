ALTER TABLE job
	ADD late_alert_sent boolean NULL,
    ADD error_alert_sent boolean NULL;

UPDATE job
SET late_alert_sent = false, error_alert_sent = false;

ALTER TABLE job
    ALTER COLUMN late_alert_sent SET NOT NULL,
    ALTER COLUMN error_alert_sent SET NOT NULL;
