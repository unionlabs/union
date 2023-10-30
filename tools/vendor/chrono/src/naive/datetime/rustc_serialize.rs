#![cfg_attr(docsrs, doc(cfg(feature = "rustc-serialize")))]

use super::NaiveDateTime;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use std::ops::Deref;

impl Encodable for NaiveDateTime {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        format!("{:?}", self).encode(s)
    }
}

impl Decodable for NaiveDateTime {
    fn decode<D: Decoder>(d: &mut D) -> Result<NaiveDateTime, D::Error> {
        d.read_str()?.parse().map_err(|_| d.error("invalid date time string"))
    }
}

/// A `DateTime` that can be deserialized from a seconds-based timestamp
#[derive(Debug)]
#[deprecated(
    since = "1.4.2",
    note = "RustcSerialize will be removed before chrono 1.0, use Serde instead"
)]
pub struct TsSeconds(NaiveDateTime);

#[allow(deprecated)]
impl From<TsSeconds> for NaiveDateTime {
    /// Pull the internal NaiveDateTime out
    #[allow(deprecated)]
    fn from(obj: TsSeconds) -> NaiveDateTime {
        obj.0
    }
}

#[allow(deprecated)]
impl Deref for TsSeconds {
    type Target = NaiveDateTime;

    #[allow(deprecated)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(deprecated)]
impl Decodable for TsSeconds {
    #[allow(deprecated)]
    fn decode<D: Decoder>(d: &mut D) -> Result<TsSeconds, D::Error> {
        Ok(TsSeconds(
            NaiveDateTime::from_timestamp_opt(d.read_i64()?, 0)
                .ok_or_else(|| d.error("invalid timestamp"))?,
        ))
    }
}

#[cfg(test)]
use rustc_serialize::json;

#[test]
fn test_encodable() {
    super::test_encodable_json(json::encode);
}

#[test]
fn test_decodable() {
    super::test_decodable_json(json::decode);
}

#[test]
fn test_decodable_timestamps() {
    super::test_decodable_json_timestamp(json::decode);
}
