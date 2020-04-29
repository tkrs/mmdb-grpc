pub mod proto;

use crate::proto::geoip2::*;
use crate::proto::geoip2_grpc::*;
use futures::Future;
use grpcio::{RpcContext, RpcStatus, RpcStatusCode, UnarySink};
use log::{debug, error};
use maxminddb::{self, geoip2, MaxMindDBError};
use spin::RwLock;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone)]
pub struct CityService<T: AsRef<[u8]>>(Arc<RwLock<maxminddb::Reader<T>>>);

impl<T: AsRef<[u8]>> CityService<T> {
    pub fn new(db: Arc<RwLock<maxminddb::Reader<T>>>) -> CityService<T> {
        CityService(db)
    }
}

impl<T: AsRef<[u8]>> GeoIp for CityService<T> {
    fn lookup(&mut self, ctx: RpcContext, req: Message, sink: UnarySink<CityReply>) {
        debug!("received the message: {:?}", req);

        let Message { ip, locales, .. } = req;
        let result = ip
            .parse()
            .map_err(|_| {
                RpcStatus::new(
                    RpcStatusCode::INVALID_ARGUMENT,
                    Some(format!("The request must be IP address but given '{}'", ip)),
                )
            })
            .and_then(|ip| (*self.0).read().lookup::<geoip2::City>(ip).map_err(convert_error))
            .map(|city| {
                debug!("found city: {:?}", city);

                let mut ns = HashSet::with_capacity(locales.len());
                for n in locales {
                    ns.insert(n.to_string());
                }
                CityReply::from(WrappedCity(city, ns))
            });

        let f = match result {
            Ok(reply) => sink.success(reply),
            Err(status) => sink.fail(status),
        };

        ctx.spawn(f.map_err(move |err| error!("failed to reply, cause: {:?}", err)))
    }
}

impl ToString for Message_Locale {
    fn to_string(&self) -> String {
        match self {
            Message_Locale::BRAZLIAN_PORTUGUESE => "pt-BR".into(),
            Message_Locale::ENGLISH => "en".into(),
            Message_Locale::FRENCH => "fr".into(),
            Message_Locale::GERMAN => "de".into(),
            Message_Locale::JAPANESE => "ja".into(),
            Message_Locale::RUSSIAN => "ru".into(),
            Message_Locale::SIMPLIFIED_CHINESE => "zh-CN".into(),
            Message_Locale::SPANISH => "es".into(),
        }
    }
}

struct WrappedCity(geoip2::City, HashSet<String>);

impl From<WrappedCity> for CityReply {
    fn from(geo_city: WrappedCity) -> CityReply {
        let mut reply = CityReply::default();

        let filter = geo_city.1;

        if let Some(c) = geo_city.0.city {
            reply.set_city(City::from(MCity(c, &filter)));
        }

        if let Some(c) = geo_city.0.continent {
            reply.set_continent(Continent::from(MContinent(c, &filter)));
        }

        if let Some(c) = geo_city.0.country {
            reply.set_country(Country::from(MCountry(c, &filter)));
        }

        if let Some(c) = geo_city.0.location {
            reply.set_location(Location::from(c));
        }

        if let Some(c) = geo_city.0.postal {
            reply.set_postal(Postal::from(c));
        }

        if let Some(c) = geo_city.0.registered_country {
            reply.set_registered_country(Country::from(MCountry(c, &filter)));
        }

        if let Some(c) = geo_city.0.represented_country {
            reply.set_represented_country(RepresentedCountry::from(MRepresentedCountry(c, &filter)));
        }

        if let Some(xs) = geo_city.0.subdivisions {
            let subs = Subdivisions::from(xs);
            let vs = ::protobuf::RepeatedField::from_vec(subs.0);
            reply.set_subdivisions(vs);
        }

        if let Some(c) = geo_city.0.traits {
            reply.set_traits(Traits::from(c));
        }

        reply
    }
}

struct MCity<'a>(geoip2::model::City, &'a HashSet<String>);

impl<'a> From<MCity<'a>> for City {
    fn from(c: MCity) -> City {
        let mut r = City::default();
        if let Some(a) = c.0.geoname_id {
            r.set_geoname_id(a);
        }
        if let Some(n) = c.0.names {
            r.set_names(filter_locales(n, c.1));
        }
        r
    }
}

struct MContinent<'a>(geoip2::model::Continent, &'a HashSet<String>);

impl<'a> From<MContinent<'a>> for Continent {
    fn from(c: MContinent) -> Continent {
        let mut r = Continent::default();
        if let Some(a) = c.0.code {
            r.set_code(a)
        }
        if let Some(a) = c.0.geoname_id {
            r.set_geoname_id(a)
        }
        if let Some(n) = c.0.names {
            r.set_names(filter_locales(n, c.1));
        }
        r
    }
}

struct MCountry<'a>(geoip2::model::Country, &'a HashSet<String>);

impl<'a> From<MCountry<'a>> for Country {
    fn from(c: MCountry) -> Country {
        let mut r = Country::default();
        if let Some(a) = c.0.geoname_id {
            r.set_geoname_id(a);
        }
        if let Some(a) = c.0.is_in_european_union {
            r.set_in_european_union(a);
        }
        if let Some(a) = c.0.iso_code {
            r.set_iso_code(a);
        }
        if let Some(n) = c.0.names {
            r.set_names(filter_locales(n, c.1));
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

struct MRepresentedCountry<'a>(geoip2::model::RepresentedCountry, &'a HashSet<String>);

impl<'a> From<MRepresentedCountry<'a>> for RepresentedCountry {
    fn from(c: MRepresentedCountry) -> RepresentedCountry {
        let mut r = RepresentedCountry::default();
        if let Some(a) = c.0.geoname_id {
            r.set_geoname_id(a);
        }
        if let Some(a) = c.0.iso_code {
            r.set_iso_code(a);
        }
        if let Some(n) = c.0.names {
            r.set_names(filter_locales(n, c.1));
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

impl From<geoip2::model::Traits> for Traits {
    fn from(c: geoip2::model::Traits) -> Traits {
        let mut t = Traits::default();
        if let Some(v) = c.is_anonymous_proxy {
            t.set_anonymous_proxy(v);
        }
        if let Some(v) = c.is_satellite_provider {
            t.set_satellite_provider(v);
        }
        t
    }
}

fn convert_error(err: MaxMindDBError) -> RpcStatus {
    match err {
        MaxMindDBError::AddressNotFoundError(msg) => RpcStatus::new(RpcStatusCode::NOT_FOUND, msg.into()),
        MaxMindDBError::InvalidDatabaseError(msg) => RpcStatus::new(RpcStatusCode::INTERNAL, msg.into()),
        MaxMindDBError::IoError(msg) => RpcStatus::new(RpcStatusCode::INTERNAL, msg.into()),
        MaxMindDBError::MapError(msg) => RpcStatus::new(RpcStatusCode::INTERNAL, msg.into()),
        MaxMindDBError::DecodingError(msg) => RpcStatus::new(RpcStatusCode::INTERNAL, msg.into()),
    }
}

fn filter_locales<'a>(names: BTreeMap<String, String>, filter: &'a HashSet<String>) -> HashMap<String, String> {
    let cap = if filter.is_empty() { names.len() } else { filter.len() };
    let mut h: HashMap<String, String> = HashMap::with_capacity(cap);
    for (k, v) in names.into_iter() {
        if filter.is_empty() || filter.contains(&k) {
            h.insert(k, v);
        }
    }
    h
}
