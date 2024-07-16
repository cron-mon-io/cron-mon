#!/usr/bin/env bash

cargo llvm-cov \
    --no-clean \
    --show-missing-lines \
    --ignore-filename-regex="src/bin/*|test_utils/*|src/infrastructure/db_schema.rs|src/infrastructure/notify/late_job_logger.rs" \
    --fail-under-lines=100 \
    -- --test-threads=1

if [ $? != 0 ]; then
    echo "100% coverage not met"
    exit 1
fi
