FROM public.ecr.aws/docker/library/rust:1.79.0-slim

RUN apt-get update && apt-get install libpq-dev -y
RUN rustup component add rustfmt
RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /usr/cron-mon/api

COPY ./api .
