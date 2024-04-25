install: build-containers npm-install migrate seed

build-containers:
	docker compose build

npm-install:
	docker compose run --rm app bash -c 'npm install'

build-api:
	docker compose run --rm --no-deps api bash -c 'cargo build --release'

build-app:
	docker compose run --rm --no-deps app bash -c  'npm run build'

run:
	docker compose up api app

# The `run-debug-deps-for-*` commands are to allow debug sessions to be run for
# the API or front-end app through the development container's debugger, while
# still running the other parts required for the whole system.
run-debug-deps-for-api:
	docker compose up app db

run-debug-deps-for-app:
	docker compose up api db

test: test-api

test-api:
	docker compose run --rm --no-deps api bash -c 'cargo test'

migration:
	docker compose run --rm api diesel migration generate $(name)

migrate:
	docker compose run --rm api diesel migration run

migrate-redo:
	docker compose run --rm api diesel migration redo

seed:
	docker compose run --rm seeder psql -f /usr/share/seeds.sql


# Can't delete the volume when PostgreSQL is running.
delete-postgres-volume:
	docker compose down db && docker volume rm cron-mon-postgres-data
