ARG RUST_VERSION=1.76.0
FROM lukemathwalker/cargo-chef:0.1.66-rust-$RUST_VERSION as chef

ENV SQLX_OFFLINE=true
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

RUN apt-get update && apt-get install build-essential protobuf-compiler python3.11-dev -y

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

FROM debian:12-slim AS runtime

LABEL org.opencontainers.image.title="Dosei"
LABEL org.opencontainers.image.description="Official Dosei image"
LABEL org.opencontainers.image.url="https://dosei.ai"
LABEL org.opencontainers.image.documentation="https://dosei.ai/docs"
LABEL org.opencontainers.image.source="https://github.com/doseiai/dosei"
LABEL org.opencontainers.image.vendor="Dosei"

RUN apt-get update && apt-get install python3.11-dev -y

WORKDIR /app
ARG RELEASE_PATH=target/release
ARG TAGET_PATH=/usr/local/bin

COPY --from=builder $RELEASE_PATH/doseid $TAGET_PATH
COPY --from=builder $RELEASE_PATH/dctl $TAGET_PATH
COPY --from=builder $RELEASE_PATH/proxy $TAGET_PATH

ENTRYPOINT ["/usr/local/bin/doseid"]
