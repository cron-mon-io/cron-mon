![Docker](https://img.shields.io/badge/Docker-2CA5E0?logo=docker&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)

[![CI](https://github.com/howamith/cron-mon/actions/workflows/ci.yml/badge.svg)](https://github.com/howamith/cron-mon/actions/workflows/ci.yml)

# Cron-Mon

Cron-Mon is a tool for monitoring cronjobs (or _tasks_ of a similar nature), built with Rust and
Vue3.

This project came about purely from my curiosity as to whether or not Rust was a good choice
of language for building web services and APIs, and how well Domain Driven Design could be applied
in that context (spoiler alert; a resounding **yes** on both counts IMO), and my desire to do
_something_ with Rust and Vue.js.

Note that there isn't anything novel about Cron-Mon; there are plenty of existing solutions in this
space already, such as [Cronitor.io](https://cronitor.io/cron-job-monitoring),
[Healthchecks.io](https://healthchecks.io), [Cronhub.io](https://cronhub.io), and
[Sentry’s Cron Monitoring](https://sentry.io/for/cron-monitoring/), to name but a few,

## Getting started

All you need to setup a development environment for Cron-Mon is Docker and your IDE of choice. A
`Makefile` is provided to make _driving_ the project quick and simple. The easiest way to get setup
is to simply clone the project and then run `make install` from the root of the project to build the
containers. From here you can run the projects (unit) tests by running `make test`, and you can run
the API application via `make run`. You'll probably want to also run `make run-monitor` in a
separate terminal/ tab to run the background service that monitors for late jobs (this is purely to
avoid cluttering the same terminal/ tab with logs from the API **and** the background serice).

A [Development container](https://containers.dev/) configuration file is also provided to ensure
your IDE can use your local containers' environments and to provide a pre-setup, consistent and
reliable debug configuration (this is tried and tested on Visual Studio Code).

### Makefile

- `install`: Builds all application containers, installs the required Node modules in the Vue
  application and sets up a local PostgreSQL database with test data.
- `build-containers`: Builds all application containers.
- `run`: Run the Cron-Mon API
- `run-monitor`: Run the Cron-Mon monitoring service.
- `test`: Run all units tests.
- `integration-tests`: Run the **integration** tests (note that this will remove whatever's in your
  local database, and as such these tests will never run unless they're invoked via this command).
- `migration`: Create a new database migration. Note that this command requires a `name` parameter,
  which be used as the name for the migration. After the migration has been created you'll need to
  write the actual migration scripts within the generated `up.sql` and `down.sql` files in the
  generated migration.
- `migrate`: Run any migrations that haven't been applied to the local database.
- `migrate-redo`: Downgrade and then re apply the latest migration on the local database.
- `seed`: Remove all data from the local database and insert the test data (this is the same test
  data that get's written to the local database during `make install`).
- `delete-postgres-volume`: Remove the Docker volume being used to make PostgreSQL data persist.
  This can be handy is you run into any problems with your local database and you just want to trash
  it and start again. The next time the database container runs this will be recreated naturally
