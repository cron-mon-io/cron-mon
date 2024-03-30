FROM node:20.12-slim

RUN apt-get update && \
    apt-get install --no-install-recommends -y \
    ca-certificates curl build-essential

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs/ | sh -s -- --default-toolchain=1.77.0 -y

ENV PATH=/root/.cargo/bin:$PATH
USER root

WORKDIR /usr/cron-mon
COPY . .