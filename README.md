# mmdb-grpc

[![crates.io](https://meritbadge.herokuapp.com/mmdb-grpc)](https://crates.io/crates/mmdb-grpc)
![](https://github.com/tkrs/mmdb-grpc/workflows/Build/badge.svg)
![](https://github.com/tkrs/mmdb-grpc/workflows/Release/badge.svg)
[![Docker Image Pulls](https://img.shields.io/docker/pulls/tkrs/mmdb-server "Docker Image Pulls")](https://img.shields.io/docker/pulls/tkrs/mmdb-server)
[![Docker Image Pulls](https://img.shields.io/docker/pulls/tkrs/mmdb-reload "Docker Image Pulls")](https://img.shields.io/docker/pulls/tkrs/mmdb-reload)

## Usage

```
❯ cargo install mmdb-grpc

...

```

```
❯ mmdb-server --help
mmdb-grpc 0.1
Takeru Sato <type.in.type@gmail.com>

USAGE:
    mmdb-server [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --host <host>                         [default: localhost]
    -f, --file <mmdb-path>                    [default: /usr/share/GeoIP/GeoLite2-City.mmdb]
    -p, --port <port>                         [default: 50000]
        --slots-per-worker <slots-per-cq>    
    -w, --workers <workers>                   [default: 1]

```

```
❯ mmdb-reload --help
mmdb-grpc 0.2.1-alpha.0
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
