FROM rust:1.74.1 as builder

ENV SQLX_OFFLINE=true

WORKDIR /usr/src/doseid

RUN apt-get update && apt-get install -y build-essential protobuf-compiler python3.11-dev

## Mock workspace
COPY Cargo.toml Cargo.lock ./

# Mock workspace members
RUN cargo new proto --lib && cargo new util --lib && cargo new doseid --bin && cargo new proxy --bin
COPY proto/Cargo.toml ./proto/
COPY doseid/Cargo.toml ./doseid/
COPY proxy/Cargo.toml ./proxy/
# Exception, for some reason doesn't work with this stuff
COPY util/ ./util/

RUN cargo build --release

COPY . .
RUN cargo build --release

FROM rust:1.74.1

RUN apt-get update && apt-get install -y python3.11-dev

ARG DOSEID_INSTALL=/bin/doseid
COPY --from=builder /usr/src/doseid/target/release/doseid ${DOSEID_INSTALL}
RUN chmod +x ${DOSEID_INSTALL}

ARG DOSEI_PROXY_INSTALL=/bin/dosei-proxy
COPY --from=builder /usr/src/doseid/target/release/proxy ${DOSEI_PROXY_INSTALL}
RUN chmod +x ${DOSEI_PROXY_INSTALL}

CMD ["/bin/doseid"]
