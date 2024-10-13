# 2. Database will be managed by the user

Date: 2024-04-10

## Status

Accepted

Also applied to Keycloak [4. keycloak will be managed by the user](0004-keycloak-managed-by-user.md)

## Context

CronMon uses a PostgreSQL database to persist Monitors and Jobs. This database needs to be managed
and maintained by *something*.

## Decision

Rather than CronMon looking after its own database, CronMon will expect users to setup and manage a
database themselves, with the only configuration required for CronMon itself being a database
connection string for CronMon to connect to.

## Consequences

Users will have to put in more effort to get CronMon setup, but since CronMon will be available in
a Docker image, users can simply use the PostgreSQL Docker image alongside it.

Putting this responsibility on the user gives them the benefit of having more control over how the
database is setup, and gives CronMon the benefit of not having to be concerned with infrastructure
concerns and focus more on the actual application.
