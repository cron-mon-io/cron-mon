# 4. keycloak will be managed by the user

Date: 2024-09-02

## Status

Accepted

Consistent with the decision around the database
[0002. Database will be managed by the user](0002-database-managed-by-user.md).

## Context

CronMon depends on a Keycloak server for JWT authentication.

## Decision

Similar to the PostgreSQL database that CronMon users that must be setup and maintained by users,
the Keycloak server that CronMon uses must be setup and maintained by the user as well. This is
consistent with previous architectural decisions made on CronMon's supporting infrastructure.

The only requirements for CronMon itself will be:

* The JWK certificates URL will need to be passed into CronMon
* The JWTs generated by the Keycloak server will need to include a `tenant` claim, who's value will
  be used by CronMon to restrict what the end-users can see to only the resources that they own.

## Consequences

Users will have to put in more effort to get CronMon setup, but since CronMon will be available in
a Docker image, users can simply use the Keycloak Docker image alongside it and setup a token mapper
for the `tenant` claim.

Putting this responsibility on the user gives them the benefit of having more control over how the
Keycloak server is setup, and gives CronMon the benefit of not having to be concerned with
infrastructure concerns and focus more on the actual application.

This decision will allow CronMon to support single-tenancy *and* multi-tenancy, since the value for
the `tenant` claim will be down to the user, they can decide what sort of tenancy to use. For
example:

* If the user is only deploying CronMon on an internal network that only those connected to said
  network have access to it, then `tenant` can be set to a hardcoded value and every end-user with
  access to the CronMon instance will be able to see every Monitor and their jobs (single-tenancy).
* If the user wants each end-user to only be able to see the Monitors that *they* have setup, then
  `tenant` can be set to something unique to the end-user, such as their email address or username.
* If the user wants to deploy CronMon publicaly and allow end-users to setup teams/ organisations,
  then the user can setup a custom registration flow in Keycloak to cater for this, and `tenant` can
  be set to the name of the team/ organisation that the end-user is part of (multi-tenancy).
