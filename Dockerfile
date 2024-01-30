FROM rust:1.74.1 as builder

ENV SQLX_OFFLINE=true

WORKDIR /usr/src/dosei

RUN apt-get update && apt-get install -y build-essential protobuf-compiler python3.11-dev

COPY . .

RUN --mount=type=cache,target=target cargo build --release

RUN mkdir release
RUN --mount=type=cache,target=target cp target/release/doseid release/doseid
RUN --mount=type=cache,target=target cp target/release/dctl release/dctl
RUN --mount=type=cache,target=target cp target/release/proxy release/proxy

FROM rust:1.74.1

RUN apt-get update && apt-get install -y python3.11-dev

ARG RELEASE_PATH=/usr/src/dosei/release
ARG DOSEID_INSTALL=/bin/doseid
ARG DOSEI_CLI_INSTALL=/bin/dctl
ARG DOSEI_PROXY_INSTALL=/bin/dosei-proxy

COPY --from=builder ${RELEASE_PATH}/doseid ${DOSEID_INSTALL}
COPY --from=builder ${RELEASE_PATH}/dctl ${DOSEI_CLI_INSTALL}
COPY --from=builder ${RELEASE_PATH}/proxy ${DOSEI_PROXY_INSTALL}

RUN chmod +x ${DOSEID_INSTALL} ${DOSEI_CLI_INSTALL} ${DOSEI_PROXY_INSTALL}

CMD ["/bin/doseid"]
