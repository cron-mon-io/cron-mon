# This Makefile runs the commands directly on the host as it's expected to be used on
# the application container or development container, purely to speed up local
# development a bit. Note that if you want to run a release build of the API or
# monitoring microservice, then you should use the Makefile at the root of this
# project on your host machine.

run-debug:
	cargo run --bin api

run-monitor-debug:
	cargo run --bin monitor

test: lint unit-test integration-tests

lint:
	cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings

unit-test:
	cargo test --lib --no-fail-fast

integration-tests:
	cargo test --no-fail-fast --jobs=5 --test "*" -- --test-threads=1

test-coverage:
	cargo llvm-cov \
        --no-clean \
        --show-missing-lines \
        --ignore-filename-regex="src/infrastructure/db_schema.rs" \
        --fail-under-lines=95 \
        --jobs=3 \
        --lcov \
        --output-path lcov.info \
        -- --test-threads=1

migration:
	diesel migration generate $(name)

migrate:
	diesel migration run

migrate-revert:
	diesel migration revert

migrate-redo:
	diesel migration redo
