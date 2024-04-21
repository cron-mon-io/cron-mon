ALTER TABLE job
	ADD max_end_time TIMESTAMP NULL;

UPDATE
	job
SET
	max_end_time = start_time + (
		SELECT
			(expected_duration + grace_duration) * INTERVAL '1 SECOND'
		FROM
			monitor
		WHERE
			monitor_id = job.monitor_id);

ALTER TABLE job
    ALTER COLUMN max_end_time SET NOT NULL;
