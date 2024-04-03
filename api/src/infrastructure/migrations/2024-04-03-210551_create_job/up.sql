CREATE TABLE job (
	job_id uuid PRIMARY KEY,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    monitor_id uuid NOT NULL,
	start_time TIMESTAMP NOT NULL,
	end_time TIMESTAMP NULL,
	status VARCHAR NULL,
	output TEXT NULL,

    CONSTRAINT fk_monitor_job
      FOREIGN KEY(monitor_id) 
        REFERENCES monitor(monitor_id)
        ON DELETE CASCADE
);

SELECT diesel_manage_updated_at('job');
