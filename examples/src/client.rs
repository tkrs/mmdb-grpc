use clap::Parser;
use grpcio::{ChannelBuilder, EnvBuilder};
use log::{error, info};
use mmdb_grpc::proto::geoip2::*;
use mmdb_grpc::proto::geoip2_grpc::GeoIpClient;
use std::sync::Arc;

#[derive(Parser)]
#[clap(version, author)]
struct Opts {
    #[clap(long = "ip", value_parser)]
    ip: String,
    #[clap(short = 'H', long = "host", value_parser, default_value = "localhost")]
    host: String,
    #[clap(short = 'P', long = "port", value_parser, default_value = "50000")]
    port: u16,
}

impl Opts {
    fn ip(&self) -> &String {
        &self.ip
    }
    fn host(&self) -> &String {
        &self.host
    }
}

fn main() {
    env_logger::init();

    let opts: Opts = Opts::parse();

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(format!("{}:{}", opts.host(), opts.port).as_ref());
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

    match client.reload(&Empty::new()) {
        Ok(r) => info!("succeeded to reload: {:?}", r),
        Err(err) => error!("failed RPC, cause: {}", err),
    }

    match client.metadata(&Empty::new()) {
        Ok(r) => info!("succeeded to request metadata: {:?}", r),
        Err(err) => error!("failed RPC, cause: {}", err),
    }
}
