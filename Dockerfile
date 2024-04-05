ARG RUST_VERSION=1.76.0
FROM lukemathwalker/cargo-chef:0.1.66-rust-$RUST_VERSION as chef

ENV SQLX_OFFLINE=true
WORKDIR /dosei

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

RUN apt-get update && apt-get install build-essential protobuf-compiler python3.11-dev -y

COPY --from=planner /dosei/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build Dosei
COPY . .
RUN cargo build --release

FROM debian:12-slim AS runtime

## Postgres
RUN apt-get update && apt-get install -y wget gnupg lsb-release

RUN sh -c 'echo "deb https://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'

RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add -

RUN apt-get update && apt-get -y install postgresql

COPY doseid/resources/pg_hba.conf /etc/postgresql/16/main/
VOLUME /var/lib/postgresql/data

LABEL org.opencontainers.image.title="Dosei"
LABEL org.opencontainers.image.description="Official Dosei image"
LABEL org.opencontainers.image.url="https://dosei.ai"
LABEL org.opencontainers.image.documentation="https://dosei.ai/docs"
LABEL org.opencontainers.image.source="https://github.com/doseiai/dosei"
LABEL org.opencontainers.image.vendor="Dosei"

# Dosei
RUN apt-get update && apt-get install python3.11-dev -y

ARG RELEASE_PATH=/dosei/target/release
ARG TAGET_PATH=/usr/local/bin

COPY --from=builder $RELEASE_PATH/doseid $TAGET_PATH
COPY --from=builder $RELEASE_PATH/dosei $TAGET_PATH
COPY --from=builder $RELEASE_PATH/proxy $TAGET_PATH

COPY entrypoint.sh /usr/local/bin/
ENTRYPOINT ["entrypoint.sh"]
