use clap::Clap;
use grpcio::{ChannelBuilder, EnvBuilder};
use mmdb_grpc::proto::geoip2::*;
use mmdb_grpc::proto::geoip2_grpc::GeoIpClient;
use std::sync::Arc;

#[derive(Clap)]
#[clap(version = "0.1", author = "Takeru Sato <type.in.type@gmail.com>")]
struct Opts {
    #[clap(long = "ip")]
    ip: String,
    #[clap(short = "h", long = "host", default_value = "localhost")]
    host: String,
    #[clap(short = "p", long = "port", default_value = "50000")]
    port: u16,
}

impl Opts {
    fn ip(&self) -> &String {
        &self.ip
    }
    fn host(&self) -> &String {
        &self.host
    }
    fn port(&self) -> u16 {
        self.port
    }
}

fn main() {
    let opts: Opts = Opts::parse();

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(format!("{}:{}", opts.host(), opts.port()).as_ref());
    let client = GeoIpClient::new(ch);

    let mut ip = Ip::default();
    ip.set_ip(opts.ip().clone());

    match client.lookup(&ip) {
        Ok(entity) => {
            println!("requested IP: {:?}, got city: {:?}", ip.ip, entity);
        }
        Err(err) => {
            println!("failed RPC, cause: {}", err);
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(1));
}
