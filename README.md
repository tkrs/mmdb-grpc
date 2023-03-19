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
mmdb-grpc x.y.z
Takeru Sato <type.in.type@gmail.com>

USAGE:
    mmdb-server [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --host <host>                                                        [default: localhost]
        --keepalive-permit-without-calls <keepalive-permit-without-calls>
        --keepalive-time <keepalive-time>
        --keepalive-timeout <keepalive-timeout>
    -f, --file <mmdb-path>
            [default: /usr/share/GeoIP/GeoLite2-City.mmdb]

    -p, --port <port>                                                        [default: 50000]
        --slots-per-worker <slots-per-worker>
    -w, --workers <workers>                                                  [default: 1]
```

```
❯ mmdb-reload --help
mmdb-grpc x.y.z
Takeru Sato <type.in.type@gmail.com>

USAGE:
    mmdb-reload [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --host <host>             [default: localhost]
    -p, --port <port>             [default: 50000]
    -s, --schedule <schedule>
```
