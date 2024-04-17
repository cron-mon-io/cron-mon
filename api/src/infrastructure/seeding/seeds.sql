-- Remove all existing data to ensure seeding doesn't fail due to unique constraints.
DELETE FROM job;
DELETE FROM monitor;

-- Monitors.
INSERT INTO monitor
    (monitor_id, "name", expected_duration, grace_duration)
VALUES
    ('c1bf0515-df39-448b-aa95-686360a33b36', 'db-backup.py',                  1800,  600),
    ('f0b291fe-bd41-4787-bc2d-1329903f7a6a', 'generate-orders.sh',            5400,  720),
    ('a04376e2-0fb5-4949-9744-7c5d0a50b411', 'init-philanges',                900,   300),
    ('309a68f1-d6a2-4312-8012-49c1b9b9af25', 'gen-manifests | send-manifest', 300,   120),
    ('0798c530-34a4-4452-b2dc-f8140fd498d5', 'bill-and-invoice',              10800, 1800);

-- Jobs.
INSERT INTO job
    (job_id, monitor_id, start_time, end_time, succeeded, "output")
VALUES
    -- db-backup.py (job in flight, failed a few days ago, otherwise all OK)
    (
        gen_random_uuid(),
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP,
        null,
        null,
        null
    ),
    (
        gen_random_uuid(),
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP - INTERVAL '23 HOURS - 29 MINUTES',
        TRUE,
        'Database successfully backed up'
    ),
    (
        gen_random_uuid(),
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '2 DAY',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY - 23 HOURS - 1 MINUTE',
        FALSE,
        'Could not connect to database'
    ),
    (
        gen_random_uuid(),
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '3 DAY',
        CURRENT_TIMESTAMP - INTERVAL '2 DAYS - 23 HOURS - 28 MINUTES',
        TRUE,
        'Database successfully backed up'
    ),
    (
        gen_random_uuid(),
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '4 DAY',
        CURRENT_TIMESTAMP - INTERVAL '3 DAYS - 23 HOURS - 32 MINUTES',
        TRUE,
        'Database successfully backed up'
    ),
    (
        gen_random_uuid(),
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '5 DAY',
        CURRENT_TIMESTAMP - INTERVAL '4 DAYS - 23 HOURS - 22 MINUTES',
        TRUE,
        'Database successfully backed up'
    ),
    -- generate-orders.sh (never managed to finish)
    (
        gen_random_uuid(),
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY',
        null,
        null,
        null
    ),
    (
        gen_random_uuid(),
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '2 DAY',
        null,
        null,
        null
    ),
    (
        gen_random_uuid(),
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '3 DAY',
        null,
        null,
        null
    ),
    (
        gen_random_uuid(),
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '4 DAY',
        null,
        null,
        null
    ),
    -- init-philanges (setup but never run)
    -- gen-manifests | send-manifest (never run successfully)
    (
        gen_random_uuid(),
        '309a68f1-d6a2-4312-8012-49c1b9b9af25',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP - INTERVAL '23 HOURS - 2 MINUTES',
        FALSE,
        'Corrupted manifest detected'
    ),
    (
        gen_random_uuid(),
        '309a68f1-d6a2-4312-8012-49c1b9b9af25',
        CURRENT_TIMESTAMP - INTERVAL '2 DAYS',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY - 23 HOURS - 43 MINUTES',
        FALSE,
        'Received SIGKILL -9'
    ),
    (
        gen_random_uuid(),
        '309a68f1-d6a2-4312-8012-49c1b9b9af25',
        CURRENT_TIMESTAMP - INTERVAL '3 DAYS',
        CURRENT_TIMESTAMP - INTERVAL '2 DAYS - 23 HOURS - 13 MINUTES',
        FALSE,
        'Timed out waiting for manifest API connection'
    ),
    -- bill-and-invoice (multiple jobs in flight)
    (
        gen_random_uuid(),
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '24 MINUTES',
        NULL,
        NULL,
        NULL
    ),
    (
        gen_random_uuid(),
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '1 HOUR - 30 MINUTES',
        NULL,
        NULL,
        NULL
    ),
    (
        gen_random_uuid(),
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '2 HOURS - 58 MINUTES',
        NULL,
        NULL,
        NULL
    ),
    (
        gen_random_uuid(),
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '3 HOURS - 55 MINUTES',
        CURRENT_TIMESTAMP - INTERVAL '6 HOURS - 25 MINUTES',
        TRUE,
        '{"bills_processed": 1234, "invoiced_generated": 325}'
    );
