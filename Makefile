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
	docker compose up --force-recreate api app

migration:
	docker compose run --rm api diesel migration generate $(name)

migrate:
	docker compose run --rm api diesel migration run

migrate-redo:
	docker compose run --rm api diesel migration redo

seed:
	docker compose run --rm seeder psql -f /usr/share/seeds.sql

delete-postgres-volume:
	# Can't delete the volume when PostgreSQL is running.
	docker compose down db && docker volume rm cron-mon-postgres-data
