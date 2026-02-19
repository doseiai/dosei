ARG RUST_VERSION=1.85.0
FROM lukemathwalker/cargo-chef:0.1.71-rust-$RUST_VERSION AS chef

ENV SQLX_OFFLINE=true
WORKDIR /dosei

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

RUN apt-get update && apt-get install build-essential python3.11-dev -y

COPY --from=planner /dosei/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build Dosei
COPY . .
RUN cargo build --release --bin doseid --bin dosei

FROM debian:12-slim AS runtime

## Postgres
RUN apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get update -o Acquire::AllowInsecureRepositories=true && \
    apt-get install -y --allow-unauthenticated wget gnupg lsb-release

RUN sh -c 'echo "deb https://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'

RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add -

RUN apt-get update && apt-get -y install postgresql-17

COPY doseid/resources/pg_hba.conf /etc/postgresql/17/main/
VOLUME /var/lib/postgresql/17/main

LABEL org.opencontainers.image.title="Dosei"
LABEL org.opencontainers.image.description="Official Dosei image"
LABEL org.opencontainers.image.url="https://dosei.io"
LABEL org.opencontainers.image.documentation="https://dosei.io/docs"
LABEL org.opencontainers.image.source="https://github.com/doseidotio/dosei"
LABEL org.opencontainers.image.vendor="Dosei"

# Dosei
RUN apt-get update && apt-get install python3.11-dev nodejs -y

ARG RELEASE_PATH=/dosei/target/release
ARG TAGET_PATH=/usr/local/bin

COPY --from=builder $RELEASE_PATH/doseid $TAGET_PATH
COPY --from=builder $RELEASE_PATH/dosei $TAGET_PATH

COPY docker-entrypoint.sh /usr/local/bin/

EXPOSE 8080
STOPSIGNAL SIGINT

ENTRYPOINT ["docker-entrypoint.sh"]
