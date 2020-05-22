FROM debian:buster-slim as builder

WORKDIR /tmp

RUN apt-get -qq update && apt-get -qq install -y curl

RUN curl -L -o grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/v0.3.2/grpc_health_probe-linux-amd64 && \
  chmod +x grpc_health_probe

FROM debian:buster-slim

ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

COPY --from=builder /tmp/grpc_health_probe /usr/local/bin/
COPY target/release/mmdb-server /usr/local/bin/

ENTRYPOINT [ "/usr/local/bin/mmdb-server" ]
