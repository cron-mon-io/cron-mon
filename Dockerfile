FROM public.ecr.aws/docker/library/rust:1.79.0-slim as builder

RUN apt-get update && apt-get install libpq-dev -y
RUN rustup component add rustfmt
RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /usr/cron-mon/api

COPY ./api .

RUN cargo build --release

FROM public.ecr.aws/docker/library/debian:bookworm-slim

# Set up a non-root user and directory
RUN groupadd -g 1000 cron-mon && \
    useradd -r -u 1000 -g cron-mon cron-mon && \
    mkdir /usr/bin/cron-mon && chown cron-mon:cron-mon /usr/bin/cron-mon
WORKDIR /usr/bin/cron-mon

RUN apt-get update && apt-get install libpq-dev -y

COPY --from=builder \
    --chown=cron-mon:cron-mon \
    /usr/cron-mon/api/diesel.toml /usr/bin/cron-mon/diesel.toml
COPY --from=builder \
    --chown=cron-mon:cron-mon \
    /usr/cron-mon/api/Rocket.toml /usr/bin/cron-mon/Rocket.toml
COPY --from=builder \
    --chown=cron-mon:cron-mon \
    /usr/cron-mon/api/docs /usr/bin/cron-mon/docs

COPY --from=builder \
    --chown=cron-mon:cron-mon \
    /usr/cron-mon/api/target/release/api /usr/bin/cron-mon/api
COPY --from=builder \
    --chown=cron-mon:cron-mon \
    /usr/cron-mon/api/target/release/monitor /usr/bin/cron-mon/monitor

USER 1000
