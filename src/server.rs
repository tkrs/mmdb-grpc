use clap::{crate_version, Clap};
use crossbeam_channel::{bounded, select, Receiver};
use futures::executor::block_on;
use grpcio::{ChannelBuilder, Environment, ServerBuilder};
use grpcio_proto::health::v1::health::*;
use log::{error, info};
use maxminddb as mmdb;
use mmdb_grpc::proto::geoip2_grpc;
use mmdb_grpc::{CityService, HealthService};
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use spin::RwLock;
use std::sync::Arc;
use std::thread;

#[derive(Clap)]
#[clap(version = crate_version!(), author = "Takeru Sato <type.in.type@gmail.com>")]
struct Opts {
    #[clap(short = 'h', long = "host", default_value = "localhost")]
    host: String,
    #[clap(short = 'p', long = "port", default_value = "50000")]
    port: u16,
    #[clap(short = 'f', long = "file", default_value = "/usr/share/GeoIP/GeoLite2-City.mmdb")]
    mmdb_path: String,
    #[clap(short = 'w', long = "workers", default_value = "1")]
    workers: usize,
    #[clap(long = "slots-per-worker")]
    slots_per_worker: Option<usize>,
    #[clap(long = "keepalive-time")]
    keepalive_time: Option<String>,
    #[clap(long = "keepalive-timeout")]
    keepalive_timeout: Option<String>,
    #[clap(long = "keepalive-permit-without-calls")]
    keepalive_permit_without_calls: Option<bool>,
}

impl Opts {
    fn mmdb_path(&self) -> &String {
        &self.mmdb_path
    }
    fn host(&self) -> &String {
        &self.host
    }
}

fn main() {
    env_logger::init();

    let opts = Opts::parse();

    let reader = mmdb::Reader::open_readfile(opts.mmdb_path()).unwrap();
    let mmdb = Arc::new(RwLock::new(reader));

    let env = Arc::new(Environment::new(opts.workers));
    let cloned_path = opts.mmdb_path().clone();
    let geoip_service = geoip2_grpc::create_geo_ip(CityService::new(mmdb.clone(), move || {
        mmdb::Reader::open_readfile(cloned_path.clone())
    }));
    let health_service = create_health(HealthService);
    let mut builder = ServerBuilder::new(env.clone())
        .register_service(geoip_service)
        .register_service(health_service)
        .bind(opts.host(), opts.port);

    let mut channel_builder = ChannelBuilder::new(env);

    if let Some(ref v) = opts.keepalive_time {
        let t = parse_duration::parse(v.as_str()).unwrap();
        channel_builder = channel_builder.keepalive_time(t);
    }
    if let Some(ref v) = opts.keepalive_timeout {
        let t = parse_duration::parse(v.as_str()).unwrap();
        channel_builder = channel_builder.keepalive_timeout(t);
    }
    if let Some(v) = opts.keepalive_permit_without_calls {
        channel_builder = channel_builder.keepalive_permit_without_calls(v)
    }

    builder = builder.channel_args(channel_builder.build_args());

    if let Some(v) = opts.slots_per_worker {
        builder = builder.requests_slot_per_cq(v);
    }

    let mut server = builder.build().unwrap();
    server.start();

    let (h, p) = server.bind_addrs().next().unwrap();
    info!("started mmdb-grpc server listening on {}:{}", h, p);

    let mmdb_path = opts.mmdb_path();
    let term_event = terminate_channel().unwrap();
    let reload_event = reload_channel().unwrap();
    loop {
        select! {
            recv(reload_event) -> _ => {
                match mmdb::Reader::open_readfile(mmdb_path.clone()) {
                    Ok(reader) => {
                        let mut db = (*mmdb).write();
                        *db = reader;
                        info!("succeeded to reload mmdb");
                    }
                    Err(err) => {
                        error!("failed to reload mmdb, cause {:?}", err);
                    }
                }
            }
            recv(term_event) -> _ => {
                info!("bye!");
                break;
            }
        }
    }

    let _ = block_on(server.shutdown());
}

fn terminate_channel() -> Result<Receiver<()>, String> {
    let (sender, receiver) = bounded(0);

    let mut signals = Signals::new(&[SIGTERM, SIGINT]).map_err(|err| err.to_string())?;

    thread::spawn(move || {
        for _ in signals.forever() {
            let _ = sender.send(());
        }
    });

    Ok(receiver)
}

fn reload_channel() -> Result<Receiver<()>, String> {
    let (sender, receiver) = bounded(0);

    let mut signals = Signals::new(&[SIGHUP]).map_err(|err| err.to_string())?;

    thread::spawn(move || loop {
        for _ in signals.wait() {
            let _ = sender.send(());
        }
    });

    Ok(receiver)
}
