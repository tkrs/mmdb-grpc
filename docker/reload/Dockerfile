FROM debian:buster-slim

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

COPY target/release/mmdb-reload /usr/local/bin/

ENV MMDB_SERVER_HOST localhost
ENV MMDB_SERVER_PORT 50000

ENTRYPOINT  [ "/usr/local/bin/mmdb-reload" ]
