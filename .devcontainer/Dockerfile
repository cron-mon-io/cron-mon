FROM node:20.12-slim

RUN apt-get update && \
    apt-get install --no-install-recommends -y \
    build-essential ca-certificates curl git libpq-dev

# Setup Rust.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs/ | sh -s -- --default-toolchain=1.77.0 -y
ENV PATH=/root/.cargo/bin:$PATH
RUN cargo install diesel_cli --no-default-features --features postgres

ENV PATH /usr/cron-mon/app/node_modules/.bin:$PATH
USER root

WORKDIR /usr/cron-mon
COPY . .

WORKDIR /usr/cron-mon/app
RUN npm install -g npm@latest
