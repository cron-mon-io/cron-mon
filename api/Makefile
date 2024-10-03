# This Makefile runs the commands directly on the host as it's expected to be used on
# the application container or development container, purely to speed up local
# development a bit. Note that if you want to run a release build of the API on
# monitoring microservice, then you should use the Makefile at the root of this
# project on your host machine.

run-debug:
	cargo run --bin api

run-monitor-debug:
	cargo run --bin monitor

test: lint unit-test

lint:
	cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings

unit-test:
	cargo test --lib --no-fail-fast

# Note that running this locally will re-seed your local DB so you'll lose
# everything in there currently.
integration-tests:
	cargo test --no-fail-fast --jobs=5 --test "*" -- --test-threads=1

# This will also re-seed your local DB, as it effectively runs *all* tests.
test-coverage:
	cargo llvm-cov \
        --no-clean \
        --show-missing-lines \
        --ignore-filename-regex="src/infrastructure/db_schema.rs" \
        --fail-under-lines=100 \
        --jobs=3 \
        -- --test-threads=1

migration:
	diesel migration generate $(name)

migrate:
	diesel migration run

migrate-revert:
	diesel migration revert

migrate-redo:
	diesel migration redo
