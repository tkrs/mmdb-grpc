# mmdb-grpc

[![crates.io](https://img.shields.io/crates/v/mmdb-grpc)](https://crates.io/crates/mmdb-grpc)
[![Build](https://github.com/tkrs/mmdb-grpc/workflows/Build/badge.svg)](https://github.com/tkrs/mmdb-grpc/actions/workflows/build.yml)
[![Release](https://github.com/tkrs/mmdb-grpc/workflows/Release/badge.svg)](https://github.com/tkrs/mmdb-grpc/actions/workflows/release.yml)

The gRPC service that provides a query to [maxminddb](https://docs.rs/crate/maxminddb/)

## Usage

```
❯ cargo install mmdb-grpc
```

```
❯ mmdb-server --help
The gRPC service that provides a query to Maxmind's GeoLite2 databases

Usage: mmdb-server [OPTIONS]

Options:
  -H, --host <HOST>
          [default: localhost]
  -P, --port <PORT>
          [default: 50000]
  -F, --file <MMDB_PATH>
          [default: /usr/share/GeoIP/GeoLite2-City.mmdb]
  -W, --workers <WORKERS>
          [default: 1]
      --slots-per-worker <SLOTS_PER_WORKER>
          
      --keepalive-time <KEEPALIVE_TIME>
          
      --keepalive-timeout <KEEPALIVE_TIMEOUT>
          
      --keepalive-permit-without-calls <KEEPALIVE_PERMIT_WITHOUT_CALLS>
          [possible values: true, false]
  -h, --help
          Print help
  -V, --version
          Print version

```

```
❯ mmdb-reload --help
Usage: mmdb-reload [OPTIONS]

Options:
  -H, --host <HOST>          [default: localhost]
  -P, --port <PORT>          [default: 50000]
  -S, --schedule <SCHEDULE>  
  -h, --help                 Print help
  -V, --version              Print version
```
