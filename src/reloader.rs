use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use cron::Schedule;
use grpcio::{ChannelBuilder, EnvBuilder, Error};
use log::{debug, info};
use mmdb_grpc::proto::geoip2::*;
use mmdb_grpc::proto::geoip2_grpc::GeoIpClient;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[clap(version, author)]
struct Opts {
    #[clap(short = 'H', long = "host", value_parser, default_value = "localhost")]
    host: String,
    #[clap(short = 'P', long = "port", value_parser, default_value = "50000")]
    port: u16,
    #[clap(short = 'S', long = "schedule", value_parser)]
    schedule: Option<String>,
}

impl Opts {
    fn host(&self) -> &String {
        &self.host
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let opts = Opts::parse();

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(format!("{}:{}", opts.host(), opts.port).as_ref());
    let client = GeoIpClient::new(ch);

    if let Some(ref expr) = opts.schedule {
        let schedule = Schedule::from_str(expr).unwrap();
        for dt in schedule.upcoming(Utc) {
            let now = Utc::now().naive_utc();
            let next = NaiveDateTime::parse_from_str(&dt.to_string(), "%Y-%m-%d %H:%M:%S UTC").unwrap();
            let delay = next - now;
            debug!("-> next reload date-time: '{}', it will take: {}", next, delay);

            let delay = Duration::new(
                delay.num_seconds() as u64,
                delay.num_nanoseconds().unwrap_or_default() as u32,
            );

            thread::sleep(delay);

            client
                .reload(&Empty::new())
                .map(|r| info!("succeeded to reload: {:?}", r))?;
        }
    } else {
        client
            .reload(&Empty::new())
            .map(|r| info!("succeeded to reload: {:?}", r))?;
    }

    Ok(())
}
