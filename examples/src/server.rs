use clap::Clap;
use crossbeam_channel::{bounded, select, Receiver};
use env_logger;
use futures::Future;
use grpcio::{Environment, ServerBuilder};
use log::{error, info};
use maxminddb as mmdb;
use mmdb_grpc::proto::geoip2_grpc;
use mmdb_grpc::CityService;
use signal_hook::{iterator::Signals, SIGHUP, SIGINT, SIGTERM};
use spin::RwLock;
use std::sync::Arc;
use std::thread;

#[derive(Clap)]
#[clap(version = "0.1", author = "Takeru Sato <type.in.type@gmail.com>")]
struct Opts {
    #[clap(short = "h", long = "host", default_value = "localhost")]
    host: String,
    #[clap(short = "p", long = "port", default_value = "50000")]
    port: u16,
    #[clap(short = "f", long = "file")]
    mmdb_path: String,
    #[clap(long = "completion-queue-count", default_value = "1")]
    completion_queue_count: usize,
    #[clap(long = "slots-per-cq")]
    slots_per_cq: Option<usize>,
}

impl Opts {
    fn mmdb_path(&self) -> &String {
        &self.mmdb_path
    }
    fn host(&self) -> &String {
        &self.host
    }
    fn port(&self) -> u16 {
        self.port
    }
    fn completion_queue_count(&self) -> usize {
        self.completion_queue_count
    }
}

fn main() {
    let _ = env_logger::init();

    let opts: Opts = Opts::parse();

    let reader = mmdb::Reader::open_readfile(opts.mmdb_path()).unwrap();
    let mmdb = Arc::new(RwLock::new(reader));

    let env = Arc::new(Environment::new(opts.completion_queue_count()));
    let service = geoip2_grpc::create_geo_ip(CityService::new(mmdb.clone()));
    let builder = ServerBuilder::new(env)
        .register_service(service)
        .bind(opts.host(), opts.port());

    let builder = if let Some(v) = opts.slots_per_cq {
        builder.requests_slot_per_cq(v)
    } else {
        builder
    };
    let mut server = builder.build().unwrap();
    server.start();

    info!("started mmdb-grpc server listening on {}:{}", opts.host(), opts.port());

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
                        break;
                    }
                }
            }
            recv(term_event) -> _ => {
                info!("bye!");
                break;
            }
        }
    }

    let _ = server.shutdown().wait();
}

fn terminate_channel() -> Result<Receiver<()>, String> {
    let (sender, receiver) = bounded(0);

    let signals = Signals::new(&[SIGTERM, SIGINT]).map_err(|err| err.to_string())?;

    thread::spawn(move || {
        for _ in signals.forever() {
            let _ = sender.send(());
        }
    });

    Ok(receiver)
}

fn reload_channel() -> Result<Receiver<()>, String> {
    let (sender, receiver) = bounded(0);

    let signals = Signals::new(&[SIGHUP]).map_err(|err| err.to_string())?;

    thread::spawn(move || loop {
        for _ in signals.forever() {
            let _ = sender.send(());
        }
    });

    Ok(receiver)
}
