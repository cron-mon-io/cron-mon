ALTER TABLE job
	ADD succeeded BOOLEAN NULL;

UPDATE
	job
SET
	succeeded = (
		CASE WHEN "status" = 'success' THEN
			TRUE
		WHEN "status" = 'error' THEN
			FALSE
		END);

ALTER TABLE job DROP "status";
