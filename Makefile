install: build-api-container

build-api-container:
	docker compose build

build-api-binary:
	docker compose run --rm --no-deps service bash -c 'cargo build --release'

run:
	docker compose up --force-recreate service
