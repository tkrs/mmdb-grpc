use futures::sync::oneshot;
use futures::Future;
use geoip_grpc::proto::geoip2_grpc;
use geoip_grpc::CityService;
use grpcio::{Environment, ServerBuilder};
use maxminddb as mmdb;
use std::io::Read;
use std::sync::Arc;
use std::{io, thread};

fn main() -> Result<(), String> {
    let env = Arc::new(Environment::new(1));
    let mut args = std::env::args().skip(1);
    let reader = mmdb::Reader::open_readfile(
        args.next()
            .ok_or_else(|| "First argument must be the path to the IP database")?,
    )
    .unwrap();
    let service = geoip2_grpc::create_geo_ip(CityService::new(reader));
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 50000)
        .build()
        .unwrap();
    server.start();
    for (host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    let _ = rx.wait();
    let _ = server.shutdown().wait();
    Ok(())
}
