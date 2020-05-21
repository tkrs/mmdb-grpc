FROM debian:buster-slim

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

COPY target/release/mmdb-reload /usr/local/bin/
COPY create-crontab /usr/local/bin/
COPY run-cron /usr/local/bin/

ENV MMDB_SERVER_HOST localhost
ENV MMDB_SERVER_PORT 50000

RUN apt-get -qq update && apt-get -qq install --no-install-recommends -y cron

ENTRYPOINT  [ "/usr/local/bin/mmdb-reload" ]
