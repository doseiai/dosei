FROM rust:1.74.1

ARG DOSEID_INSTALL=/bin/doseid
ARG DOSEI_PROXY_INSTALL=/bin/dosei-proxy
ENV SQLX_OFFLINE=true

WORKDIR /usr/src/doseid

RUN apt-get update && apt-get install protobuf-compiler --yes

# Mock workspace
COPY Cargo.toml Cargo.lock ./

# Mock workspace members
RUN cargo new proto --lib && cargo new doseid --bin && cargo new proxy --bin && cargo new util --lib
COPY proto/Cargo.toml ./proto/
COPY doseid/Cargo.toml ./doseid/
COPY proxy/Cargo.toml ./proxy/
COPY util/Cargo.toml ./util/

RUN cargo build --release

COPY . .
RUN cargo build --release

RUN mv target/release/doseid ${DOSEID_INSTALL}
RUN chmod +x ${DOSEID_INSTALL}

RUN mv target/release/proxy ${DOSEI_PROXY_INSTALL}
RUN chmod +x ${DOSEI_PROXY_INSTALL}

RUN rm -rf target

CMD ["/bin/doseid"]
