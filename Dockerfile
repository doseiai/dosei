FROM rust:1.74.1

ARG DOSEI_INSTALL=/bin/dosei
ARG DOSEI_PROXY_INSTALL=/bin/dosei-proxy
ENV SQLX_OFFLINE=true

WORKDIR /usr/src/dosei

RUN apt-get update && apt-get install protobuf-compiler --yes

COPY . .

RUN cargo build --release

RUN mv target/release/dosei ${DOSEI_INSTALL}
RUN chmod +x ${DOSEI_INSTALL}

RUN mv target/release/proxy ${DOSEI_PROXY_INSTALL}
RUN chmod +x ${DOSEI_PROXY_INSTALL}

RUN rm -rf target

CMD ["/bin/dosei"]
