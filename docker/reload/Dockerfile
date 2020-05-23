FROM debian:buster-slim

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

COPY target/release/mmdb-reload /usr/local/bin/

ENTRYPOINT [ "/usr/local/bin/mmdb-reload" ]
