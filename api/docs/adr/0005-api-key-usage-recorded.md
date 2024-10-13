# 5. API key usage will be recorded

Date: 2024-10-13

## Status

Accepted



## Context

Starting and finishing jobs cannot be authenticated with CronMon's JWT authentication, since these
JWT's could be short lived (this is ultimately up to the user and how they've setup their Keycloak
server), which would force end-users' integrations with CronMon to go through the OIDC login flow to
obtain a JWT. While this is perfectly possible, it does make integrating with CronMon far more
complex than it needs to be. To mitigate this, API keys will be used instead to authenticate
starting and finishing jobs. But, since an API key is valid indefinitely (unless the end-user
deletes it), this means this form of authentication is a lot less secure than JWTs.

## Decision

Everytime an API key is used to access a monitor to either start or finish a Job, it's usage will
be recorded, including the time at which it was used, and the monitor it was used for. This
information can then be retrievable via the API, allowing end-users to see when their API keys are
used. This means that should an end-user suspect that one of their API keys has become comprimised,
they could use this information to see if it's being as expected.

## Consequences

Because we'll need access to the API key not just when authenticating the request, but also once
we've retrieved the Monitor, so that we can record the usage of the API key against that monitor, we
won't be able to create a simple
[Request guard](https://rocket.rs/guide/v0.5/requests/#request-guards) here to take care of all
aspects of API key auth. Instead, we'll also need to bring in some of this into the actual
CronMon domain.
