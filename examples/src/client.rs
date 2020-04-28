use std::sync::Arc;

use grpcio::{ChannelBuilder, EnvBuilder};

use geoip_grpc::proto::geoip2::*;
use geoip_grpc::proto::geoip2_grpc::GeoIpClient;

fn main() {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(format!("localhost:{}", 50000).as_str());
    let client = GeoIpClient::new(ch);

    loop {
        let mut ip = Ip::default();
        ip.set_ip("126.203.22.11".into());

        match client.lookup(&ip) {
            Ok(entity) => {
                println!("{:?}, city: {:?}", ip, entity);
            }
            Err(err) => {
                println!("Failed RPC, cause: {}", err);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
