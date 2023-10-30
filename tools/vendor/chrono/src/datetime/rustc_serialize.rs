#![cfg_attr(docsrs, doc(cfg(feature = "rustc-serialize")))]

use super::DateTime;
#[cfg(feature = "clock")]
use crate::offset::Local;
use crate::offset::{FixedOffset, LocalResult, TimeZone, Utc};
use core::fmt;
use core::ops::Deref;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

impl<Tz: TimeZone> Encodable for DateTime<Tz> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        format!("{:?}", self).encode(s)
    }
}

// lik? function to convert a LocalResult into a serde-ish Result
fn from<T, D>(me: LocalResult<T>, d: &mut D) -> Result<T, D::Error>
where
    D: Decoder,
    T: fmt::Display,
{
    match me {
        LocalResult::None => Err(d.error("value is not a legal timestamp")),
        LocalResult::Ambiguous(..) => Err(d.error("value is an ambiguous timestamp")),
        LocalResult::Single(val) => Ok(val),
    }
}

impl Decodable for DateTime<FixedOffset> {
    fn decode<D: Decoder>(d: &mut D) -> Result<DateTime<FixedOffset>, D::Error> {
        d.read_str()?.parse::<DateTime<FixedOffset>>().map_err(|_| d.error("invalid date and time"))
    }
}

#[allow(deprecated)]
impl Decodable for TsSeconds<FixedOffset> {
    #[allow(deprecated)]
    fn decode<D: Decoder>(d: &mut D) -> Result<TsSeconds<FixedOffset>, D::Error> {
        from(FixedOffset::east_opt(0).unwrap().timestamp_opt(d.read_i64()?, 0), d).map(TsSeconds)
    }
}

impl Decodable for DateTime<Utc> {
    fn decode<D: Decoder>(d: &mut D) -> Result<DateTime<Utc>, D::Error> {
        d.read_str()?
            .parse::<DateTime<FixedOffset>>()
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| d.error("invalid date and time"))
    }
}

/// A [`DateTime`] that can be deserialized from a timestamp
///
/// A timestamp here is seconds since the epoch
#[derive(Debug)]
pub struct TsSeconds<Tz: TimeZone>(DateTime<Tz>);

#[allow(deprecated)]
impl<Tz: TimeZone> From<TsSeconds<Tz>> for DateTime<Tz> {
    /// Pull the inner `DateTime<Tz>` out
    #[allow(deprecated)]
    fn from(obj: TsSeconds<Tz>) -> DateTime<Tz> {
        obj.0
    }
}

#[allow(deprecated)]
impl<Tz: TimeZone> Deref for TsSeconds<Tz> {
    type Target = DateTime<Tz>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(deprecated)]
impl Decodable for TsSeconds<Utc> {
    fn decode<D: Decoder>(d: &mut D) -> Result<TsSeconds<Utc>, D::Error> {
        from(Utc.timestamp_opt(d.read_i64()?, 0), d).map(TsSeconds)
    }
}

#[cfg(feature = "clock")]
#[cfg_attr(docsrs, doc(cfg(feature = "clock")))]
impl Decodable for DateTime<Local> {
    fn decode<D: Decoder>(d: &mut D) -> Result<DateTime<Local>, D::Error> {
        match d.read_str()?.parse::<DateTime<FixedOffset>>() {
            Ok(dt) => Ok(dt.with_timezone(&Local)),
            Err(_) => Err(d.error("invalid date and time")),
        }
    }
}

#[cfg(feature = "clock")]
#[cfg_attr(docsrs, doc(cfg(feature = "clock")))]
#[allow(deprecated)]
impl Decodable for TsSeconds<Local> {
    #[allow(deprecated)]
    fn decode<D: Decoder>(d: &mut D) -> Result<TsSeconds<Local>, D::Error> {
        from(Utc.timestamp_opt(d.read_i64()?, 0), d).map(|dt| TsSeconds(dt.with_timezone(&Local)))
    }
}

#[cfg(test)]
use rustc_serialize::json;

#[test]
fn test_encodable() {
    super::test_encodable_json(json::encode, json::encode);
}

#[cfg(feature = "clock")]
#[test]
fn test_decodable() {
    super::test_decodable_json(json::decode, json::decode, json::decode);
}

#[cfg(feature = "clock")]
#[test]
fn test_decodable_timestamps() {
    super::test_decodable_json_timestamps(json::decode, json::decode, json::decode);
}
