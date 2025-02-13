FROM public.ecr.aws/docker/library/rust:1.83-slim as builder

RUN apt-get update && apt-get install build-essential libpq-dev libssl-dev pkg-config -y
RUN rustup component add rustfmt clippy
RUN cargo install diesel_cli --no-default-features --features postgres && \
    cargo install cargo-llvm-cov adrs

WORKDIR /usr/cron-mon/api

COPY ./api .

RUN cargo build --release

FROM public.ecr.aws/docker/library/debian:bookworm-slim

# Set up a non-root user and directory
RUN groupadd -g 1000 cron-mon && \
    useradd -r -u 1000 -g cron-mon cron-mon && \
    mkdir /usr/bin/cron-mon && chown cron-mon:cron-mon /usr/bin/cron-mon
WORKDIR /usr/bin/cron-mon

RUN apt-get update && apt-get install ca-certificates libpq-dev -y \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

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
    /usr/cron-mon/api/target/release/cron-mon /usr/bin/cron-mon/cron-mon

USER 1000
