FROM rust:1.74.1

ARG DOSEI_INSTALL=/bin/dosei
ENV SQLX_OFFLINE=true

WORKDIR /usr/src/dosei

RUN apt-get update && apt-get install protobuf-compiler --yes

COPY . .

RUN cargo build --release

RUN mv target/release/dosei ${DOSEI_INSTALL}
RUN chmod +x ${DOSEI_INSTALL}

CMD ["/bin/dosei"]
