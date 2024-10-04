ALTER TABLE monitor
	ADD tenant VARCHAR NULL;

UPDATE
	monitor
SET
	tenant = 'cron-mon-io/cron-mon';

ALTER TABLE monitor
    ALTER COLUMN tenant SET NOT NULL;

-- The combination of monitor_id and tenant should be unique.
ALTER TABLE monitor
    ADD CONSTRAINT key_monitor_monitor_id_tenant UNIQUE (monitor_id, tenant);

-- Add indexes on the tenant column and the combination of monitor_id and tenant columns.
CREATE INDEX idx_monitor_tenant ON monitor(tenant);
CREATE INDEX idx_monitor_monitor_id_tenant ON monitor(monitor_id, tenant);
