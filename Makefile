install: build-containers migrate seed

build-containers:
	docker compose build

build-api:
	docker compose run --rm --no-deps api bash -c 'cargo build --release'

run:
	docker compose up api

run-monitor:
	docker compose up monitor

test:
	docker compose run --rm --no-deps api bash -c 'cargo test --lib --no-fail-fast'

# Note that running this locally will re-seed your local DB so you'll lose
# everything in there currently.
integration-tests:
	docker compose up integration-tests-rs

migration:
	docker compose run --rm api diesel migration generate $(name)

migrate:
	docker compose run --rm api diesel migration run

migrate-redo:
	docker compose run --rm api diesel migration redo

seed:
	docker compose run --rm seeder


# Can't delete the volume when PostgreSQL is running.
delete-postgres-volume:
	docker compose down db && docker volume rm cron-mon-postgres-data
