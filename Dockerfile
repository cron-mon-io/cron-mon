FROM public.ecr.aws/docker/library/rust:1.77.0-slim as dev

RUN apt-get update && apt-get install libpq-dev -y
RUN rustup component add rustfmt
RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /usr/cron-mon/api

COPY ./api .

FROM dev as builder

RUN cargo build --release

FROM public.ecr.aws/docker/library/alpine:3.19 as runner

# Set up a non-root user and directory
RUN addgroup -g 10001 -S cron-mon && \
    adduser -SDH -u 10001 cron-mon && \
    mkdir /bin/cron-mon && chown cron-mon:cron-mon /bin/cron-mon

COPY --from=builder \
    --chown=cron-mon:cron-mon \
    /usr/cron-mon/api/target/release/api /bin/cron-mon/api
COPY --from=builder \
    --chown=cron-mon:cron-mon \
    /usr/cron-mon/api/target/release/monitor /bin/cron-mon/monitor
