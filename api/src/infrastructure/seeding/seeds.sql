-- Remove all existing data to ensure seeding doesn't fail due to unique constraints.
DELETE FROM job;
DELETE FROM monitor;
DELETE FROM api_key;

-- Monitors.
INSERT INTO monitor
    (monitor_id, tenant, "name", expected_duration, grace_duration)
VALUES
    ('c1bf0515-df39-448b-aa95-686360a33b36', 'cron-mon', 'db-backup.py',                  1800,  600),
    ('f0b291fe-bd41-4787-bc2d-1329903f7a6a', 'cron-mon', 'generate-orders.sh',            5400,  720),
    ('a04376e2-0fb5-4949-9744-7c5d0a50b411', 'cron-mon', 'init-philanges',                900,   300),
    ('309a68f1-d6a2-4312-8012-49c1b9b9af25', 'cron-mon', 'gen-manifests | send-manifest', 300,   120),
    ('0798c530-34a4-4452-b2dc-f8140fd498d5', 'cron-mon', 'bill-and-invoice',              10800, 1800);

-- Jobs.
INSERT INTO job
    (job_id, monitor_id, start_time, max_end_time, end_time, succeeded, "output", late_alert_sent, error_alert_sent)
VALUES
    -- db-backup.py (job in flight, failed a few days ago, otherwise all OK)
    (
        '8106bab7-d643-4ede-bd92-60c79f787344',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS',
        null,
        null,
        null,
        FALSE,
        FALSE
    ),
    (
        'c1893113-66d7-4707-9a51-c8be46287b2c',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP - INTERVAL '23 HOURS 29 MINUTES',
        TRUE,
        'Database successfully backed up',
        FALSE,
        FALSE
    ),
    (
        '9d4e2d69-af63-4c1e-8639-60cb2683aee5',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '2 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '2 DAY',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY 23 HOURS 1 MINUTE',
        FALSE,
        'Could not connect to database',
        FALSE,
        TRUE
    ),
    (
        '0d66d5ec-d5b7-4f35-ab06-5e82ec17da66',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '3 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '3 DAY',
        CURRENT_TIMESTAMP - INTERVAL '2 DAYS 23 HOURS 28 MINUTES',
        TRUE,
        'Database successfully backed up',
        FALSE,
        FALSE
    ),
    (
        '28c7c0ce-c78c-4bd4-98c4-33c8d5362e3e',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '4 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '4 DAY',
        CURRENT_TIMESTAMP - INTERVAL '3 DAYS 23 HOURS 32 MINUTES',
        TRUE,
        'Database successfully backed up',
        FALSE,
        FALSE
    ),
    (
        '6c9f7a93-9418-40de-a2c3-94ed46c22161',
        'c1bf0515-df39-448b-aa95-686360a33b36',
        CURRENT_TIMESTAMP - INTERVAL '5 DAY',
        CURRENT_TIMESTAMP + INTERVAL '2400 SECONDS' - INTERVAL '5 DAY',
        CURRENT_TIMESTAMP - INTERVAL '4 DAYS 23 HOURS 22 MINUTES',
        TRUE,
        'Database successfully backed up',
        FALSE,
        FALSE
    ),
    -- generate-orders.sh (never managed to finish)
    (
        '2a09c819-ed8c-4e3a-b085-889f3f475c02',
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY',
        CURRENT_TIMESTAMP + INTERVAL '6120 SECONDS' - INTERVAL '1 DAY',
        null,
        null,
        null,
        TRUE,
        FALSE
    ),
    (
        'db610603-5094-49a4-8838-204103cd5b78',
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '2 DAY',
        CURRENT_TIMESTAMP + INTERVAL '6120 SECONDS' - INTERVAL '2 DAY',
        null,
        null,
        null,
        TRUE,
        FALSE
    ),
    (
        '7122b6cb-4910-40e6-abb8-c7558c0aae99',
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '3 DAY',
        CURRENT_TIMESTAMP + INTERVAL '6120 SECONDS' - INTERVAL '3 DAY',
        null,
        null,
        null,
        TRUE,
        FALSE
    ),
    (
        '4f4c6846-9fb7-45fa-bedd-84a381426fb0',
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        CURRENT_TIMESTAMP - INTERVAL '4 DAY',
        CURRENT_TIMESTAMP + INTERVAL '6120 SECONDS' - INTERVAL '4 DAY',
        null,
        null,
        null,
        TRUE,
        FALSE
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
        'Corrupted manifest detected',
        FALSE,
        TRUE
    ),
    (
        '8f438139-6251-466b-99a3-ce30690660aa',
        '309a68f1-d6a2-4312-8012-49c1b9b9af25',
        CURRENT_TIMESTAMP - INTERVAL '2 DAYS',
        CURRENT_TIMESTAMP + INTERVAL '420 SECONDS' - INTERVAL '2 DAYS',
        CURRENT_TIMESTAMP - INTERVAL '1 DAY 23 HOURS 43 MINUTES',
        FALSE,
        'Received SIGKILL -9',
        FALSE,
        TRUE
    ),
    (
        'b1f446a4-c3a2-4840-af49-55d54ed1989c',
        '309a68f1-d6a2-4312-8012-49c1b9b9af25',
        CURRENT_TIMESTAMP - INTERVAL '3 DAYS',
        CURRENT_TIMESTAMP + INTERVAL '420 SECONDS' - INTERVAL '3 DAYS',
        CURRENT_TIMESTAMP - INTERVAL '2 DAYS 23 HOURS 13 MINUTES',
        FALSE,
        'Timed out waiting for manifest API connection',
        FALSE,
        TRUE
    ),
    -- bill-and-invoice (multiple jobs in flight)
    (
        'eaf19950-efff-4d26-bc0e-b855b361a5ba',
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '24 MINUTES',
        CURRENT_TIMESTAMP + INTERVAL '12600 SECONDS' - INTERVAL '24 MINUTES',
        NULL,
        NULL,
        NULL,
        FALSE,
        FALSE
    ),
    (
        '4428ff3c-516a-41c9-bd62-78512b305d62',
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '1 HOUR 30 MINUTES',
        CURRENT_TIMESTAMP + INTERVAL '12600 SECONDS' - INTERVAL '1 HOUR 30 MINUTES',
        NULL,
        NULL,
        NULL,
        FALSE,
        FALSE
    ),
    (
        '3814c0f3-a9e6-47b6-b387-a61ddb4d9c2d',
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '2 HOURS 58 MINUTES',
        CURRENT_TIMESTAMP + INTERVAL '12600 SECONDS' - INTERVAL '2 HOURS 58 MINUTES',
        NULL,
        NULL,
        NULL,
        FALSE,
        FALSE
    ),
    (
        'c9576731-7650-4957-9cc7-aace50506402',
        '0798c530-34a4-4452-b2dc-f8140fd498d5',
        CURRENT_TIMESTAMP - INTERVAL '3 HOURS 55 MINUTES',
        CURRENT_TIMESTAMP + INTERVAL '12600 SECONDS' - INTERVAL '3 HOURS 55 MINUTES',
        CURRENT_TIMESTAMP - INTERVAL '6 HOURS 25 MINUTES',
        TRUE,
        '{"bills_processed": 1234, "invoiced_generated": 325}',
        FALSE,
        FALSE
    );

-- API keys.
INSERT INTO api_key
    (api_key_id, tenant, name, key, masked, last_used, last_used_monitor_id, last_used_monitor_name)
VALUES
    (
        '270e1d61-baf2-4f29-a04f-eee956da8f9e',
        'cron-mon',
        'Cron Mon API Key',
        -- Real key is 'PVb97yWJEu43OgMnVMC9BOi4CnXlPbww'
        'd13793a1c8e33f36a6bb6dd3457b41446933f001b9cada8b9c9a222836550ee7',
        'PVb97************XlPbww',
        NULL,
        NULL,
        NULL
    ),
    (
        '58c8e622-cabc-4799-80ef-5fac91601ce2',
        'cron-mon',
        'cron-mon-py API Key',
        -- Real key is 'FTkM4cTNHRRLAusMrD5r23LFzdi5mVKP'
        '62594892624f92be9f6753b2255427d1a3e4159bd8b007f6412baa3674d424ad',
        'FTkM4************di5mVKP',
        CURRENT_TIMESTAMP - INTERVAL '90 DAYS',
        'f0b291fe-bd41-4787-bc2d-1329903f7a6a',
        'generate-orders.sh'
    );
