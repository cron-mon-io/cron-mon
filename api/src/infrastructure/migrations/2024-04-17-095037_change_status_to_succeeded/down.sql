ALTER TABLE job
	ADD status VARCHAR NULL;

UPDATE
	job
SET
	"status" = (
		CASE WHEN succeeded = TRUE THEN
			'success'
		WHEN succeeded = FALSE THEN
			'error'
		END);

ALTER TABLE job DROP succeeded;
