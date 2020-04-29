use clap::Clap;
use env_logger;
use grpcio::{ChannelBuilder, EnvBuilder};
use log::{error, info};
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
    let _ = env_logger::init();

    let opts: Opts = Opts::parse();

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(format!("{}:{}", opts.host(), opts.port()).as_ref());
    let client = GeoIpClient::new(ch);

    let mut msg = Message::default();
    msg.set_ip(opts.ip().clone());

    match client.lookup(&msg) {
        Ok(entity) => {
            info!("requested message: {:?}, got city: {:?}", msg, entity);
        }
        Err(err) => {
            error!("failed RPC, cause: {}", err);
        }
    }

    msg.set_locales(vec![Message_Locale::ENGLISH, Message_Locale::JAPANESE]);

    match client.lookup(&msg) {
        Ok(entity) => {
            info!("requested message: {:?}, got city: {:?}", msg, entity);
        }
        Err(err) => {
            error!("failed RPC, cause: {}", err);
        }
    }
}
