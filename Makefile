install: build-containers migrate seed

build-containers:
	docker compose build

run:
	docker compose up api

run-debug:
	docker compose up api-debug

run-monitor:
	docker compose up monitor

run-monitor-debug:
	docker compose up monitor-debug

test: lint unit-test

lint:
	docker compose run --rm --no-deps rust-cargo make lint

unit-test:
	docker compose run --rm --no-deps rust-cargo make unit-test

# Note that running this locally will re-seed your local DB so you'll lose
# everything in there currently.
integration-tests:
	docker compose up integration-tests-rs

# This will also re-seed your local DB, as it effectively runs *all* tests.
test-coverage:
	docker compose run --rm --no-deps rust-cargo make test-coverage

migration:
	docker compose run --rm rust-cargo make migration name=$(name)

migrate:
	docker compose run --rm rust-cargo make migrate

migrate-revert:
	docker compose run --rm rust-cargo make migrate-revert

migrate-redo:
	docker compose run --rm rust-cargo make migrate-redo

seed:
	docker compose run --rm seeder

shell:
	docker compose run --rm rust-cargo bash


# Can't delete the volume when PostgreSQL is running.
delete-postgres-volume:
	docker compose down db && docker volume rm cron-mon-postgres-data
