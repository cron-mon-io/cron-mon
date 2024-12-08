# 6. Alert Configurations will use CTI (Class Table Inheritance)

Date: 2024-11-29

## Status

Accepted



## Context

The issue motivating this decision, and any context that influences or constrains the decision.

Cron Mon will allow users to configure multiple types of alerts to be triggered when jobs are late or exhibit an error. This data will be polymorphic, since it will consist of attributes common to all types of alerts, and attributes that are specific to specific alert types.

Since this data, like the rest of the data Cron Mon uses and produces, will be persisted in a PostgreSQL database, this means there are several options for representing this data in SQL:

1. **Single table inheritance** - a single SQL table with nullable columns for the data that is specific to different alert types. Simple but could get unruly if we end up supporting lots of different alert types, and adds some complexity in forcing alert-specific columns to be nullable.
2. **Concrete table inheritance** - dedicated SQL tables for each alert type to be configured, with duplicate columns in each table for the common attributes. Avoids the problems with single table inheritance, but in a trade off for new problems - namely; duplicate columns, no record on which columns are the common columns, and querying for all alert configurations would require the use of unions.
3. **Class table inheritance** - a single base table, with derivative tables for each specific alert type that gets combined with the base table to form the whole model. Introduces a bit of complexity in that adding new alert configurations requires multiple `INSERT`s, and querying for alert configuration data requires `JOIN`s, but the benefits of using class table inheritance are ultimately solving the problems associated with the two previous options, and stronger typing that is much more representative of the domain model itself
4. **Single table with a JSON column** - a single table with a JSON-type column to contain all of the alert-specific data. A simpler approach to class table inheritance, with the trade off of losing some of our contraints and increasing the complexity around model validation, since the JSON column can contain any valid JSON string

## Decision

**TL;DR: Alert configuration data will use class table inheritance**, since this allows for non-null contraints over mandatory data, easily distinguisable common attributes and alert-specific attributes, single queries to fetch all alert configurations (albeit with `JOIN`s), with the tradeoff that multiple tables must be written to for updates and data integrity must be manually enforced by the application when doing so (i.e a _foo_ alert must have a record in both the base table and the _foo_-specific table).

Option 1 is ruled out on the grounds that we could potentially end up with lots of supported alert integrations. For example, we're looking at Slack as the first integration, followed by Webhooks and possibly Discord. Should this project take off though and there is appetitate for it, this could well be expanded to include the likes of email, SMS, MS Teams, Signal etc. There is a counter argument here that we're only looking at Slack and webhooks until there is need for anything else, but since we know this _could_ be a problem if we _do_ end up with lots of alert integrations, it makes sense to design for the future here as best we can, rather than opt for something we already know might need to be rewritten - not to mention requiring careful migration of users' data. Note that this option would also allow for invalid data to be written to the database, due to alert-specific columns being nullable, which we also really want to avoid.

Option 2 is ruled out on the grounds that querying for all alert configurations will require multiple queries with unions, which is less optimal than a single query as can be achieved with class table inheritance, as well as the overly repetative table structure than options 3 and 4.

Option 4 is ruled out on the basis that it adds too much complexity and opportunity for error from introducing a JSON column, whose contents would need to be verified for each row being returned.

## Consequences

Rust does not support inheritance, so modelling the data in our database into a domain model will not be an exact mapping from SQL to Rust code, as it might be with a language that _does_ support inheritance.

> [!NOTE]
> This will likely involve an `Enum` in our Rust code.
