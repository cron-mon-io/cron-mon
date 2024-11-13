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

test:
	docker compose run --rm --no-deps rust-cargo make test

lint:
	docker compose run --rm --no-deps rust-cargo make lint

unit-test:
	docker compose run --rm --no-deps rust-cargo make unit-test

integration-tests:
	docker compose run --rm --no-deps rust-cargo make integration-test

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
