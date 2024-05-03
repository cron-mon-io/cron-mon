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
    (job_id, monitor_id, start_time, max_end_time, end_time, succeeded, "output")
VALUES
    -- db-backup.py (job in flight, failed a few days ago, otherwise all OK)
    (
        '8106bab7-d643-4ede-bd92-60c79f787344',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS',
        null,
        null,
        null
    ),
    (
        'c1893113-66d7-4707-9a51-c8be46287b2c',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP - INTERVAL '23 HOURS 29 MINUTES',
        TRUE,
        'Database successfully backed up'
    ),
    (
        '9d4e2d69-af63-4c1e-8639-60cb2683aee5',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '2 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '2 DAY',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY 23 HOURS 1 MINUTE',
        FALSE,
        'Could not connect to database'
    ),
    (
        '0d66d5ec-d5b7-4f35-ab06-5e82ec17da66',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '3 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '3 DAY',
        CURRENT_TIMESTAMP - INTERVAL '2 DAYS 23 HOURS 28 MINUTES',
        TRUE,
        'Database successfully backed up'
    ),
    (
        '28c7c0ce-c78c-4bd4-98c4-33c8d5362e3e',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '4 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '4 DAY',
        CURRENT_TIMESTAMP - INTERVAL '3 DAYS 23 HOURS 32 MINUTES',
        TRUE,
        'Database successfully backed up'
    ),
    (
        '6c9f7a93-9418-40de-a2c3-94ed46c22161',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '5 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '5 DAY',
        CURRENT_TIMESTAMP - INTERVAL '4 DAYS 23 HOURS 22 MINUTES',
        TRUE,
        'Database successfully backed up'
    ),
    -- generate-orders.sh (never managed to finish)
    (
        '2a09c819-ed8c-4e3a-b085-889f3f475c02',
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP + INTERVAL '6120 SECONDS' - INTERVAL '1 DAY',
        null,
        null,
        null
    ),
    (
        'db610603-5094-49a4-8838-204103cd5b78',
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '2 DAY',
        CURRENT_TIMESTAMP + INTERVAL '6120 SECONDS' - INTERVAL '2 DAY',
        null,
        null,
        null
    ),
    (
        '7122b6cb-4910-40e6-abb8-c7558c0aae99',
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '3 DAY',
        CURRENT_TIMESTAMP + INTERVAL '6120 SECONDS' - INTERVAL '3 DAY',
        null,
        null,
        null
    ),
    (
        '4f4c6846-9fb7-45fa-bedd-84a381426fb0',
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '4 DAY',
        CURRENT_TIMESTAMP + INTERVAL '6120 SECONDS' - INTERVAL '4 DAY',
        null,
        null,
        null
    ),
    -- init-philanges (setup but never run)
    -- gen-manifests | send-manifest (never run successfully)
    (
        '596558eb-d4ba-4aa4-a1f7-3d23d8a1b461',
        '309a68f1-d6a2-4312-8012-49c1b9b9af25',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP + INTERVAL '420 SECONDS' - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP - INTERVAL '23 HOURS 2 MINUTES',
        FALSE,
        'Corrupted manifest detected'
    ),
    (
        '8f438139-6251-466b-99a3-ce30690660aa',
        '309a68f1-d6a2-4312-8012-49c1b9b9af25',
        CURRENT_TIMESTAMP - INTERVAL '2 DAYS',
        CURRENT_TIMESTAMP + INTERVAL '420 SECONDS' - INTERVAL '2 DAYS',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY 23 HOURS 43 MINUTES',
        FALSE,
        'Received SIGKILL -9'
    ),
    (
        'b1f446a4-c3a2-4840-af49-55d54ed1989c',
        '309a68f1-d6a2-4312-8012-49c1b9b9af25',
        CURRENT_TIMESTAMP - INTERVAL '3 DAYS',
        CURRENT_TIMESTAMP + INTERVAL '420 SECONDS' - INTERVAL '3 DAYS',
        CURRENT_TIMESTAMP - INTERVAL '2 DAYS 23 HOURS 13 MINUTES',
        FALSE,
        'Timed out waiting for manifest API connection'
    ),
    -- bill-and-invoice (multiple jobs in flight)
    (
        'eaf19950-efff-4d26-bc0e-b855b361a5ba',
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '24 MINUTES',
        CURRENT_TIMESTAMP + INTERVAL '12600 SECONDS' - INTERVAL '24 MINUTES',
        NULL,
        NULL,
        NULL
    ),
    (
        '4428ff3c-516a-41c9-bd62-78512b305d62',
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '1 HOUR 30 MINUTES',
        CURRENT_TIMESTAMP + INTERVAL '12600 SECONDS' - INTERVAL '1 HOUR 30 MINUTES',
        NULL,
        NULL,
        NULL
    ),
    (
        '3814c0f3-a9e6-47b6-b387-a61ddb4d9c2d',
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '2 HOURS 58 MINUTES',
        CURRENT_TIMESTAMP + INTERVAL '12600 SECONDS' - INTERVAL '2 HOURS 58 MINUTES',
        NULL,
        NULL,
        NULL
    ),
    (
        'c9576731-7650-4957-9cc7-aace50506402',
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '3 HOURS 55 MINUTES',
        CURRENT_TIMESTAMP + INTERVAL '12600 SECONDS' - INTERVAL '3 HOURS 55 MINUTES',
        CURRENT_TIMESTAMP - INTERVAL '6 HOURS 25 MINUTES',
        TRUE,
        '{"bills_processed": 1234, "invoiced_generated": 325}'
    );
