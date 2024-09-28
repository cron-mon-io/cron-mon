![CronMon Logo](.github/assets/logo.svg)

# A simple tool for monitoring cronjobs

[![CI](https://github.com/cron-mon-io/cron-mon/actions/workflows/ci.yml/badge.svg)](https://github.com/cron-mon-io/cron-mon/actions/workflows/ci.yml)
![Coverage](https://img.shields.io/badge/coverage-100%25-green)
![Docker](https://img.shields.io/badge/Docker-2CA5E0?logo=docker&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Beta](https://img.shields.io/badge/Status-beta-blue)

CronMon is a tool for monitoring cronjobs (or _tasks_ of a similar nature), written in Rust. It was
created for two reasons:

1. Curiosity as to whether or not Rust was a good choice of language for building web services and
   APIs, and how well Domain Driven Design fits with the language.
2. A need to monitor numerous cronjobs, but nothing to justify spending money on one of the
   pre-existing soloutions.

Further to the second point, note that there isn't anything novel about CronMon; there are plenty of
existing solutions in this space already, such as
[Cronitor.io](https://cronitor.io/cron-job-monitoring), [Healthchecks.io](https://healthchecks.io),
[Cronhub.io](https://cronhub.io), and
[Sentry’s Cron Monitoring](https://sentry.io/for/cron-monitoring/), to name but a few.

## Current status

CronMon is currently still under development, but it does support basic usage.

## Basic design

CronMon is designed around _Monitors_ and _Jobs_. A Monitor is responsible for monitoring the
execution of a specific cronjob or task and reporting any issues or failures, and _contains_ Jobs,
which are a record of a single execution of the cronjob or task being monitored.

CronMon is comprised of two key components; a RESTful API for creating and managing Monitors, and
recording their Jobs; and a microservice that detects when jobs are late and notifies the owner of
the Monitor.

> Note that CronMon currently doesn't have the ability to notify when Jobs are late, but this is
> coming soon.

There is also a frontend application for CronMon, which you'll find at
https://github.com/cron-mon-io/cron-mon-app.

## Getting started

All you need to setup a development environment for CronMon is Docker and your IDE of choice. A
`Makefile` is provided to make _driving_ the project quick and simple. The easiest way to get setup
is to simply clone the project and then run `make install` from the root of the project to build the
containers. From here you can run the projects (unit) tests by running `make test`, and you can run
the API application via `make run`, after which the API will be available to you locally at
`http://127.0.0.1:8000`, which you'll be able to confirm by hitting
[the healthcheck endpoint](http://127.0.0.1:8000/api/v1/health).

![Running the API](.github/assets/getting-started.gif)

You'll probably want to also run `make run-monitor` in a separate terminal/ tab to run the
background service that monitors for late jobs (this is purely to avoid cluttering the same
terminal/ tab with logs from the API **and** the background serice). When this is running you should
see something similar to this in the terminal its running in.

![Running the monitor](.github/assets/run-monitor.gif)

A [Development container](https://containers.dev/) configuration file is also provided to ensure
your IDE can use your local containers' environments and to provide a pre-setup, consistent and
reliable debug configuration (this is tried and tested on Visual Studio Code).

### Makefiles

There are two `Makefile`s provided to make local development a bit easier; one at the root of the
project, intended to be ran on your host machine, running through Docker; and another within `/api`,
intended to be ran within the application container of development container.

Both `Makefile`s have mostly the same commands, with the exception of the following commands that
only the root-level `Makefile` has:

- `install`: Builds all application containers, installs the required Node modules in the Vue
  application and sets up a local PostgreSQL database with test data.
- `build-containers`: Builds all application containers.
- `seed`: Remove all data from the local database and insert the test data (this is the same test
  data that get's written to the local database during `make install`).
- `shell`: Open a `bash` shell on the application container, where you can use the _other_
  `Makefile` to run commands without the overhead of spinning up containers for each command.
- `delete-postgres-volume`: Remove the Docker volume being used to make PostgreSQL data persist.
  This can be handy is you run into any problems with your local database and you just want to trash
  it and start again. The next time the database container runs this will be recreated naturally

The following commands are present in both `Makefile`s:

- `run`: Run the CronMon API (release build).
- `run-debug` Run a debug build of the CronMon API.
- `run-monitor`: Run the CronMon monitoring service (release build).
- `run-monitor-debug`: Run a debug build of the CronMon monitoring service.
- `test`: Run all linting checks and _unit_ tests.
- `lint`: Run linting checks (utimately `cargo fmt` and `clippy`).
- `unit-test`: Run all _unit_ tests.
- `integration-tests`: Run the **integration** tests (note that this will remove whatever's in your
  local database, and as such these tests will never run unless they're invoked via this command).
- `test-coverage`: Run **all** tests (_unit tests and integration tests_) and get test coverage,
  ensuring we have 100% test coverage.
- `migration`: Create a new database migration. Note that this command requires a `name` parameter,
  which be used as the name for the migration. After the migration has been created you'll need to
  write the actual migration scripts within the generated `up.sql` and `down.sql` files in the
  generated migration.
- `migrate`: Run any migrations that haven't been applied to the local database.
- `migrate-revert`: Downgrade the latest migration on the local database.
- `migrate-redo`: Downgrade and then re apply the latest migration on the local database.

## Deployment

CronMon currently isn't deployed anywhere, but this may change in the future. However, its container
images are available for use on GitHub Container Registry, at `ghcr.io/cron-mon-io/cron-mon`.

If you do what to deploy CronMon, there are a couple of prerequisite requirements:

> ![INFO]
> It is a conscious design decision for CronMon not to manage its own infrastructural dependencies.
> Whilst it's acknowledged that this puts more onus on the user, we believe that the benefits
> outweigh the drawbacks here, since it gives users far more control over how not just CronMon is
> setup and deployed, but also it's supporting infrastructure. It also allows for faster CronMon
> development in offloading infrastructure concerns outside of CronMon's responsibility.

### Data Persistence

CronMon uses a Postgres database for data persistence, so you'll need to setup a Postgres instance
and expose the connection string in an environment variables called `DATABASE_URL`. From here,
CronMon will handle setting up the required tables and performing migrations, but you will have to
handle database upgrades and general maintenance yourself.

### Authentication

CronMon uses Keycloak for JWT authentication, so you'll also need to setup a Keycloak server, which
requires a little bit more work than the Postgres database. The only configuration CronMon requires
itself for this is to expose the OpenID Connect certificate URL in an environment variable called
`KEYCLOAK_CERTS_URL`. The rest of the configuration lies within Keyclaok itself:

1. Create a token mapper that includes `cron-mon` as an audience via the `aud` claim. This allows
   you to use client roles from different clients whilst still identifying tokens as intended for
   CronMon. If you get authentication errors with `AuthenticationError("InvalidAudience")` errors in
   the logs, then this mapper is missing or setup incorrectly.
2. Create a token mapper that adds a `tenant` claim. The value that should be contained here depends
   on if you need multi-tenancy or single-tenancy. More on this in the next section.

#### Multi and Single Tenancy

CronMon supports both single-tenant and multi-tenant authentication. This is controlled via the
`tenant` claim in the JWT, which is ultimately used to decide who _owns_ Monitors when they are
created, and which Monitors end-users can see when viewing them.

Single-tenancy is the most simple to setup - simply setup a token mapper in the Keycloak admin
console that sets the value to something unique about each user, such as their email address or
username.

Multi-tenancy requires a little bit more work, but still requires a token mapper, only this time the
value of the `tenant` claim should be set to something that groups users together. Exactly how users
are grouped is up to the user, and must also be implemented by them. For example, you could simply
take the domain of the end-user's email address during signup, which would be relatively straight
forward. On the other end of the spectrum, you could setup custom registration flows within Keycloak
and custom SPIs to allow for _teams_ or _organisations_ to be setup and members invited to them.
Exactly what users do here is entirely up to them - as is the choice between single and multi
tenancy.

> ![INFO]
> This is a good example of the benefits, that come with CronMon not managing it's own supporting
infrastructure, that outweigh the drawbacks!
