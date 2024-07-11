#!/usr/bin/env bash

REQUIRED_THRESHOLD='100.00%'

rm -rf target/coverage

CARGO_INCREMENTAL=0 \
RUSTFLAGS='-Cinstrument-coverage' \
LLVM_PROFILE_FILE='target/coverage/profraw/cargo-test-%p-%m.profraw' \
    cargo test -- --test-threads=1

grcov target/coverage/profraw \
    --binary-path ./target/debug/deps/ -s . -t markdown --branch \
    --ignore-not-existing \
    --ignore "../*" \
    --ignore "/*" \
    --ignore "target/*" \
    --ignore "tests/*" \
    --ignore "test_utils/*" \
    --ignore "src/bin/*" \
    --ignore "src/infrastructure/db_schema.rs" \
    --ignore "src/infrastructure/notify/late_job_logger.rs" \
    --excl-line "#\[rocket::launch\]" \
    --excl-start "#\[cfg\(test\)\]" \
    --excl-br-start "#\[cfg\(test\)\]" \
    -o target/coverage/grcov.md

cat target/coverage/grcov.md
coverage=$(cat target/coverage/grcov.md | grep "Total coverage: " | grep -o -E "[+-]?([0-9]*[.])?[0-9]+")
cov_num=$(echo $coverage | tr -d '.')
threshold=$(echo $REQUIRED_THRESHOLD | tr -d '.%')
if (( "$cov_num" < "$threshold" )); then
    echo "Coverage $coverage% is less than required threshold ($REQUIRED_THRESHOLD%)"
    exit 1
fi
