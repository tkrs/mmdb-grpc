FROM rust:1.43.1 as builder

WORKDIR /tmp

RUN apt-get -qq update && apt-get -qq install -y cmake binutils-dev llvm clang curl unzip golang libunwind-dev
RUN curl -OL https://github.com/google/protobuf/releases/download/v3.11.4/protoc-3.11.4-linux-x86_64.zip && \
  unzip protoc-3.11.4-linux-x86_64.zip -d protoc3 && \
  mv protoc3/bin/* /usr/local/bin/ && \
  mv protoc3/include/* /usr/local/include/ && \
  protoc --version

RUN rustup component add clippy

RUN mkdir -p /tmp/workspace/project/src
WORKDIR /tmp/workspace/project

COPY Cargo.toml .
COPY build.rs .
COPY protos protos

RUN mkdir src/proto
RUN echo "fn main() {}" > src/reloader.rs

RUN cargo build --release --bin mmdb-reload

COPY . .

RUN cargo clippy --release --bin mmdb-reload -- -D warnings
RUN cargo build --release --bin mmdb-reload

FROM debian:buster-slim

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

COPY --from=builder /tmp/workspace/project/target/release/mmdb-reload /usr/local/bin/
COPY --from=builder /tmp/workspace/project/create-crontab /usr/local/bin/
COPY --from=builder /tmp/workspace/project/run-cron /usr/local/bin/

ENV MMDB_SERVER_HOST localhost
ENV MMDB_SERVER_PORT 50000

RUN apt-get -qq update && apt-get -qq install --no-install-recommends -y cron

ENTRYPOINT  [ "/usr/local/bin/mmdb-reload" ]
