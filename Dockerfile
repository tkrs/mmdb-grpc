FROM rust:1.43.1 as stage

WORKDIR /tmp

RUN apt-get -qq update && apt-get -qq install -y cmake binutils-dev llvm clang curl unzip golang libunwind-dev
RUN curl -L -o grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/v0.3.2/grpc_health_probe-linux-amd64 && \
  chmod +x grpc_health_probe
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
RUN echo "fn main() {}" > src/server.rs
# RUN echo "use protoc_grpcio\nfn main() {}" > build.rs

RUN cargo clippy --release --bin mmdb-server -- -D warnings
RUN cargo build --release --bin mmdb-server

COPY . .

RUN cargo build --release --bin mmdb-server

FROM debian:buster-slim

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

COPY --from=stage /tmp/workspace/project/target/release/mmdb-server /usr/local/bin/
COPY --from=stage /tmp/grpc_health_probe /usr/local/bin/

ENTRYPOINT  [ "/usr/local/bin/mmdb-server" ]
