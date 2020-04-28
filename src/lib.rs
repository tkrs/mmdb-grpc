pub mod proto;

use crate::proto::geoip2::*;
use crate::proto::geoip2_grpc::*;
use futures::Future;
use grpcio::{RpcContext, RpcStatus, RpcStatusCode, UnarySink};
use maxminddb::{self, geoip2, MaxMindDBError};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct CityService<T: AsRef<[u8]>>(Arc<maxminddb::Reader<T>>);

impl<T: AsRef<[u8]>> CityService<T> {
    pub fn new(db: maxminddb::Reader<T>) -> CityService<T> {
        CityService(Arc::new(db))
    }
}

impl<T: AsRef<[u8]>> GeoIp for CityService<T> {
    fn lookup(&mut self, ctx: RpcContext, req: Ip, sink: UnarySink<CityReply>) {
        if let Some(ip) = req.ip.parse().ok() {
            match (*self.0).lookup::<geoip2::City>(ip) {
                Ok(a) => {
                    let reply = CityReply::from(a);
                    let f = sink
                        .success(reply.clone())
                        .map_err(move |err| eprintln!("failed to reply: {:?}", err));
                    ctx.spawn(f)
                }
                Err(err) => {
                    let status = convert_error(err);
                    let f = sink
                        .fail(status)
                        .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
                    ctx.spawn(f)
                }
            }
        } else {
            let f = sink
                .fail(RpcStatus::new(
                    RpcStatusCode::INVALID_ARGUMENT,
                    Some(format!(
                        "The request must be IP address but given '{}'",
                        req.ip
                    )),
                ))
                .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
            ctx.spawn(f)
        }
    }
}
impl From<geoip2::model::City> for City {
    fn from(c: geoip2::model::City) -> City {
        let mut r = City::default();
        if let Some(a) = c.geoname_id {
            r.set_geoname_id(a);
        }
        if let Some(n) = c.names {
            let mut h: HashMap<String, String> = HashMap::with_capacity(n.len());
            for (k, v) in n.into_iter() {
                h.insert(k, v);
            }
            r.set_names(h);
        }
        r
    }
}

impl From<geoip2::model::Continent> for Continent {
    fn from(c: geoip2::model::Continent) -> Continent {
        let mut r = Continent::default();
        if let Some(a) = c.code {
            r.set_code(a)
        }
        if let Some(a) = c.geoname_id {
            r.set_geoname_id(a)
        }
        if let Some(n) = c.names {
            let mut h: HashMap<String, String> = HashMap::with_capacity(n.len());
            for (k, v) in n.into_iter() {
                h.insert(k, v);
            }
            r.set_names(h);
        }
        r
    }
}

impl From<geoip2::model::Country> for Country {
    fn from(c: geoip2::model::Country) -> Country {
        let mut r = Country::default();
        if let Some(a) = c.geoname_id {
            r.set_geoname_id(a);
        }
        if let Some(a) = c.is_in_european_union {
            r.set_is_in_european_union(a);
        }
        if let Some(a) = c.iso_code {
            r.set_iso_code(a);
        }
        if let Some(n) = c.names {
            let mut h: HashMap<String, String> = HashMap::with_capacity(n.len());
            for (k, v) in n.into_iter() {
                h.insert(k, v);
            }
            r.set_names(h);
        }
        r
    }
}

impl From<geoip2::model::Location> for Location {
    fn from(c: geoip2::model::Location) -> Location {
        let mut r = Location::default();
        if let Some(a) = c.latitude {
            r.set_latitude(a);
        }
        if let Some(a) = c.longitude {
            r.set_longitude(a);
        }
        if let Some(a) = c.metro_code {
            r.set_metro_code(a as u32);
        }
        if let Some(a) = c.time_zone {
            r.set_time_zone(a);
        }
        r
    }
}

impl From<geoip2::model::Postal> for Postal {
    fn from(c: geoip2::model::Postal) -> Postal {
        let mut r = Postal::default();
        if let Some(a) = c.code {
            r.set_code(a);
        }
        r
    }
}

impl From<geoip2::model::RepresentedCountry> for RepresentedCountry {
    fn from(c: geoip2::model::RepresentedCountry) -> RepresentedCountry {
        let mut r = RepresentedCountry::default();
        if let Some(a) = c.geoname_id {
            r.set_geoname_id(a);
        }
        if let Some(a) = c.iso_code {
            r.set_iso_code(a);
        }
        if let Some(n) = c.names {
            let mut h: HashMap<String, String> = HashMap::with_capacity(n.len());
            for (k, v) in n.into_iter() {
                h.insert(k, v);
            }
            r.set_names(h);
        }
        r
    }
}

#[derive(PartialEq, Clone, Default)]
struct Subdivisions(Vec<Subdivision>);

impl From<Vec<geoip2::model::Subdivision>> for Subdivisions {
    fn from(vs: Vec<geoip2::model::Subdivision>) -> Subdivisions {
        let mut subs = Vec::with_capacity(vs.len());

        for s in vs {
            let mut sub = Subdivision::default();
            if let Some(v) = s.geoname_id {
                sub.set_geoname_id(v);
            }
            if let Some(v) = s.iso_code {
                sub.set_iso_code(v);
            }
            subs.push(sub);
        }
        Subdivisions(subs)
    }
}

impl From<geoip2::City> for CityReply {
    fn from(geo_city: geoip2::City) -> CityReply {
        let mut reply = CityReply::default();
        if let Some(c) = geo_city.city {
            reply.set_city(City::from(c));
        }

        if let Some(c) = geo_city.country {
            reply.set_country(Country::from(c));
        }

        if let Some(xs) = geo_city.subdivisions {
            let subs = Subdivisions::from(xs);
            let vs = ::protobuf::RepeatedField::from_vec(subs.0);
            reply.set_subdivisions(vs);
        }

        reply
    }
}

fn convert_error(err: MaxMindDBError) -> RpcStatus {
    match err {
        MaxMindDBError::AddressNotFoundError(msg) => {
            RpcStatus::new(RpcStatusCode::NOT_FOUND, msg.into())
        }
        MaxMindDBError::InvalidDatabaseError(msg) => {
            RpcStatus::new(RpcStatusCode::INTERNAL, msg.into())
        }
        MaxMindDBError::IoError(msg) => RpcStatus::new(RpcStatusCode::INTERNAL, msg.into()),
        MaxMindDBError::MapError(msg) => RpcStatus::new(RpcStatusCode::INTERNAL, msg.into()),
        MaxMindDBError::DecodingError(msg) => RpcStatus::new(RpcStatusCode::INTERNAL, msg.into()),
    }
}
