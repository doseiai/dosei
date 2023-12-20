FROM rust:1.74.1

ARG DOSEID_INSTALL=/bin/doseid
ARG DOSEI_PROXY_INSTALL=/bin/dosei-proxy
ENV SQLX_OFFLINE=true

WORKDIR /usr/src/doseid

RUN apt-get update && apt-get install protobuf-compiler --yes

COPY Cargo.toml Cargo.lock ./

# Mock proto
RUN cargo new proto --lib
COPY proto/Cargo.toml /proto

# Mock doseid
RUN cargo new doseid --bin
COPY doseid/Cargo.toml /doseid

# Mock proxy
RUN cargo new proxy --bin
COPY proxy/Cargo.toml /proxy

RUN cargo build --release

COPY . .
RUN cargo build --release

RUN mv target/release/doseid ${DOSEID_INSTALL}
RUN chmod +x ${DOSEID_INSTALL}

RUN mv target/release/proxy ${DOSEI_PROXY_INSTALL}
RUN chmod +x ${DOSEI_PROXY_INSTALL}

RUN rm -rf target

CMD ["/bin/doseid"]
