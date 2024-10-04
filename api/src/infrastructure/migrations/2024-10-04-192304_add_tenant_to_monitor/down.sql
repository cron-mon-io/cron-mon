-- Drop the indexes and constraints on the tenant column.
DROP INDEX idx_monitor_tenant;
DROP INDEX idx_monitor_monitor_id_tenant;
ALTER TABLE monitor
    DROP CONSTRAINT key_monitor_monitor_id_tenant;

-- Drop the tenant column.
ALTER TABLE monitor DROP tenant;
