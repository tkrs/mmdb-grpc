use clap::Clap;
use grpcio::{ChannelBuilder, EnvBuilder, Error};
use log::info;
use mmdb_grpc::proto::geoip2::*;
use mmdb_grpc::proto::geoip2_grpc::GeoIpClient;
use std::sync::Arc;

#[derive(Clap)]
#[clap(version = "0.1", author = "Takeru Sato <type.in.type@gmail.com>")]
struct Opts {
    #[clap(short = "h", long = "host", default_value = "localhost")]
    host: String,
    #[clap(short = "p", long = "port", default_value = "50000")]
    port: u16,
}

impl Opts {
    fn host(&self) -> &String {
        &self.host
    }
    fn port(&self) -> u16 {
        self.port
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let opts: Opts = Opts::parse();

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(format!("{}:{}", opts.host(), opts.port()).as_ref());
    let client = GeoIpClient::new(ch);

    client
        .reload(&Empty::new())
        .map(|r| info!("succeeded to reload: {:?}", r))
}
