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
	docker compose run --rm --no-deps api-debug bash -c 'cargo test --lib --no-fail-fast'

# Note that running this locally will re-seed your local DB so you'll lose
# everything in there currently.
integration-tests:
	docker compose up integration-tests-rs

migration:
	docker compose run --rm api-debug diesel migration generate $(name)

migrate:
	docker compose run --rm api diesel migration run

migrate-redo:
	docker compose run --rm api diesel migration redo

seed:
	docker compose run --rm seeder


# Can't delete the volume when PostgreSQL is running.
delete-postgres-volume:
	docker compose down db && docker volume rm cron-mon-postgres-data
