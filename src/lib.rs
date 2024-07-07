pub mod proto;

use crate::proto::geoip2::*;
use crate::proto::geoip2_grpc::*;
use futures::prelude::*;
use grpcio::{RpcContext, RpcStatus, RpcStatusCode, UnarySink};
use grpcio_health::proto::*;
use log::{debug, error};
use maxminddb::{self, geoip2, MaxMindDBError, Metadata};
use spin::RwLock;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Display;
use std::sync::Arc;

#[derive(Clone)]
pub struct CityService<T, R>(Arc<RwLock<maxminddb::Reader<T>>>, R)
where
    T: AsRef<[u8]>,
    R: Fn() -> Result<maxminddb::Reader<T>, MaxMindDBError>;

impl<T, R> CityService<T, R>
where
    T: AsRef<[u8]>,
    R: Fn() -> Result<maxminddb::Reader<T>, MaxMindDBError>,
{
    pub fn new(db: Arc<RwLock<maxminddb::Reader<T>>>, reloader: R) -> CityService<T, R> {
        CityService(db, reloader)
    }
}

impl<T, R> GeoIp for CityService<T, R>
where
    T: AsRef<[u8]>,
    R: Fn() -> Result<maxminddb::Reader<T>, MaxMindDBError>,
{
    fn lookup(&mut self, ctx: RpcContext<'_>, req: Message, sink: UnarySink<CityReply>) {
        debug!("received the message: {:?}", req);

        let Message { ip, locales, .. } = req;
        let result = ip
            .parse()
            .map_err(|_| {
                RpcStatus::with_message(
                    RpcStatusCode::INVALID_ARGUMENT,
                    format!("The request must be IP address but given '{}'", ip),
                )
            })
            .and_then(|ip| {
                let db = (*self.0).read();
                match db.lookup::<geoip2::City>(ip) {
                    Ok(value) => {
                        let ns = locales.iter().map(|l| l.to_string()).collect::<HashSet<_>>();
                        Ok(CityReply::from(WrappedCity(value, ns)))
                    }
                    Err(err) => Err(convert_error(err)),
                }
            });

        let f = match result {
            Ok(reply) => sink.success(reply),
            Err(status) => sink.fail(status),
        };

        let f = f
            .map_err(move |err| error!("failed to reply, cause: {:?}", err))
            .map(|_| ());

        ctx.spawn(f)
    }

    fn metadata(&mut self, ctx: RpcContext<'_>, _req: Empty, sink: UnarySink<MetadataReply>) {
        let result = MetadataReply::from(&self.0.read().metadata);
        let f = sink
            .success(result)
            .map_err(move |err| error!("failed to reply, cause: {:?}", err))
            .map(|_| ());
        ctx.spawn(f)
    }

    fn reload(&mut self, ctx: RpcContext<'_>, _req: Empty, sink: UnarySink<MetadataReply>) {
        let result = self.1()
            .map(|reader| {
                let mut guard = self.0.write();
                *guard = reader;
                MetadataReply::from(&guard.metadata)
            })
            .map_err(convert_error);

        let f = match result {
            Ok(reply) => sink.success(reply),
            Err(status) => sink.fail(status),
        };

        let f = f
            .map_err(move |err| error!("failed to reply, cause: {:?}", err))
            .map(|_| ());

        ctx.spawn(f)
    }
}

impl Display for Message_Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            Message_Locale::BRAZLIAN_PORTUGUESE => "pt-BR",
            Message_Locale::ENGLISH => "en",
            Message_Locale::FRENCH => "fr",
            Message_Locale::GERMAN => "de",
            Message_Locale::JAPANESE => "ja",
            Message_Locale::RUSSIAN => "ru",
            Message_Locale::SIMPLIFIED_CHINESE => "zh-CN",
            Message_Locale::SPANISH => "es",
            Message_Locale::UNSPECIFIED => "", // TODO: should it panic?
        };
        f.write_str(data)
    }
}

struct WrappedCity<'a>(geoip2::City<'a>, HashSet<String>);

impl<'a> From<WrappedCity<'a>> for CityReply {
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

struct MCity<'a>(geoip2::city::City<'a>, &'a HashSet<String>);

impl<'a> From<MCity<'a>> for City {
    fn from(c: MCity) -> City {
        let mut r = City::default();
        if let Some(a) = c.0.geoname_id {
            r.set_geoname_id(a);
        }
        if let Some(n) = c.0.names {
            r.set_names(filter_locales(&n, c.1));
        }
        r
    }
}

struct MContinent<'a>(geoip2::city::Continent<'a>, &'a HashSet<String>);

impl<'a> From<MContinent<'a>> for Continent {
    fn from(c: MContinent) -> Continent {
        let mut r = Continent::default();
        if let Some(a) = c.0.code {
            r.set_code(a.to_string())
        }
        if let Some(a) = c.0.geoname_id {
            r.set_geoname_id(a)
        }
        if let Some(n) = c.0.names {
            r.set_names(filter_locales(&n, c.1));
        }
        r
    }
}

struct MCountry<'a>(geoip2::city::Country<'a>, &'a HashSet<String>);

impl<'a> From<MCountry<'a>> for Country {
    fn from(c: MCountry) -> Country {
        let mut r = Country::default();
        if let Some(a) = c.0.geoname_id {
            r.set_geoname_id(a);
        }
        if let Some(a) = c.0.is_in_european_union {
            r.is_in_european_union = a;
        }
        if let Some(a) = c.0.iso_code {
            r.set_iso_code(a.to_string());
        }
        if let Some(n) = &c.0.names {
            r.set_names(filter_locales(n, c.1)); // TODO: clone
        }
        r
    }
}

impl<'a> From<geoip2::city::Location<'a>> for Location {
    fn from(c: geoip2::city::Location) -> Location {
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
            r.set_time_zone(a.to_string());
        }
        r
    }
}

impl<'a> From<geoip2::city::Postal<'a>> for Postal {
    fn from(c: geoip2::city::Postal) -> Postal {
        let mut r = Postal::default();
        if let Some(a) = c.code {
            r.set_code(a.to_string());
        }
        r
    }
}

struct MRepresentedCountry<'a>(geoip2::city::RepresentedCountry<'a>, &'a HashSet<String>);

impl<'a> From<MRepresentedCountry<'a>> for RepresentedCountry {
    fn from(c: MRepresentedCountry) -> RepresentedCountry {
        let mut r = RepresentedCountry::default();
        if let Some(a) = c.0.geoname_id {
            r.set_geoname_id(a);
        }
        if let Some(a) = c.0.iso_code {
            r.set_iso_code(a.to_string());
        }
        if let Some(n) = c.0.names {
            r.set_names(filter_locales(&n, c.1));
        }
        r
    }
}

#[derive(PartialEq, Clone, Default)]
struct Subdivisions(Vec<Subdivision>);

impl<'a> From<Vec<geoip2::city::Subdivision<'a>>> for Subdivisions {
    fn from(vs: Vec<geoip2::city::Subdivision>) -> Subdivisions {
        let mut subs = Vec::with_capacity(vs.len());

        for s in vs {
            let mut sub = Subdivision::default();
            if let Some(v) = s.geoname_id {
                sub.set_geoname_id(v);
            }
            if let Some(v) = s.iso_code {
                sub.set_iso_code(v.to_string());
            }
            subs.push(sub);
        }
        Subdivisions(subs)
    }
}

impl From<geoip2::city::Traits> for Traits {
    fn from(c: geoip2::city::Traits) -> Traits {
        let mut t = Traits::default();
        if let Some(v) = c.is_anonymous_proxy {
            t.is_anonymous_proxy = v;
        }
        if let Some(v) = c.is_satellite_provider {
            t.is_satellite_provider = v;
        }
        t
    }
}

fn convert_error(err: MaxMindDBError) -> RpcStatus {
    match err {
        MaxMindDBError::AddressNotFoundError(msg) => RpcStatus::with_message(RpcStatusCode::NOT_FOUND, msg),
        MaxMindDBError::InvalidNetworkError(msg) => RpcStatus::with_message(RpcStatusCode::INTERNAL, msg),
        MaxMindDBError::InvalidDatabaseError(msg) => RpcStatus::with_message(RpcStatusCode::INTERNAL, msg),
        MaxMindDBError::IoError(msg) => RpcStatus::with_message(RpcStatusCode::INTERNAL, msg),
        MaxMindDBError::MapError(msg) => RpcStatus::with_message(RpcStatusCode::INTERNAL, msg),
        MaxMindDBError::DecodingError(msg) => RpcStatus::with_message(RpcStatusCode::INTERNAL, msg),
    }
}

fn filter_locales<'a>(names: &'a BTreeMap<&'a str, &'a str>, filter: &'a HashSet<String>) -> HashMap<String, String> {
    let cap = if filter.is_empty() { names.len() } else { filter.len() };
    let mut h = HashMap::with_capacity(cap);
    for (k, v) in names.iter() {
        let k = k.to_string();
        if filter.is_empty() || filter.contains(&k) {
            h.insert(k, v.to_string());
        }
    }
    h
}

impl From<&Metadata> for MetadataReply {
    fn from(v: &Metadata) -> MetadataReply {
        let mut r = MetadataReply::default();
        r.set_binary_format_major_version(v.binary_format_major_version as u32);
        r.set_binary_format_minor_version(v.binary_format_minor_version as u32);
        r.set_build_epoch(v.build_epoch);
        r.set_database_type(v.database_type.clone());
        let d =
            v.description
                .clone()
                .into_iter()
                .fold(HashMap::with_capacity(v.description.len()), |mut acc, (k, v)| {
                    acc.insert(k, v);
                    acc
                });
        r.set_description(d);
        r.set_ip_version(v.ip_version as u32);
        r.set_languages(::protobuf::RepeatedField::from_vec(v.languages.clone()));
        r.set_node_count(v.node_count);
        r.set_record_size(v.record_size as u32);
        r
    }
}

#[derive(Clone)]
pub struct HealthService;

impl Health for HealthService {
    fn check(&mut self, ctx: RpcContext<'_>, req: HealthCheckRequest, sink: UnarySink<HealthCheckResponse>) {
        debug!("check the service: {}", req.get_service());
        let mut resp = HealthCheckResponse::default();
        resp.set_status(ServingStatus::Serving);
        ctx.spawn(
            sink.success(resp)
                .map_err(|e| error!("failed to report result: {:?}", e))
                .map(|_| ()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_locales() {
        let mut src = BTreeMap::new();
        src.insert("1", "one");
        src.insert("2", "two");
        src.insert("3", "three");
        src.insert("4", "four");

        let mut filters = HashSet::new();
        filters.insert("11".to_string());
        filters.insert("2".to_string());
        filters.insert("3".to_string());
        let actual = filter_locales(&src, &filters);

        let mut expected = HashMap::new();
        expected.insert("2".to_string(), "two".to_string());
        expected.insert("3".to_string(), "three".to_string());
        assert_eq!(actual, expected);

        let filters = HashSet::new();
        let actual = filter_locales(&src, &filters);

        let mut expected = HashMap::new();
        expected.insert("1".to_string(), "one".to_string());
        expected.insert("2".to_string(), "two".to_string());
        expected.insert("3".to_string(), "three".to_string());
        expected.insert("4".to_string(), "four".to_string());
        assert_eq!(actual, expected);
    }
}
