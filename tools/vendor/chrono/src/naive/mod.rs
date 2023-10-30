//! Date and time types unconcerned with timezones.
//!
//! They are primarily building blocks for other types
//! (e.g. [`TimeZone`](../offset/trait.TimeZone.html)),
//! but can be also used for the simpler date and time handling.

mod date;
pub(crate) mod datetime;
mod internals;
mod isoweek;
mod time;

#[allow(deprecated)]
pub use self::date::{Days, NaiveDate, NaiveWeek, MAX_DATE, MIN_DATE};
#[cfg(feature = "rustc-serialize")]
#[allow(deprecated)]
pub use self::datetime::rustc_serialize::TsSeconds;
#[allow(deprecated)]
pub use self::datetime::{NaiveDateTime, MAX_DATETIME, MIN_DATETIME};
pub use self::isoweek::IsoWeek;
pub use self::time::NaiveTime;

#[cfg(feature = "__internal_bench")]
#[doc(hidden)]
pub use self::internals::YearFlags as __BenchYearFlags;

/// Serialization/Deserialization of naive types in alternate formats
///
/// The various modules in here are intended to be used with serde's [`with`
/// annotation][1] to serialize as something other than the default [RFC
/// 3339][2] format.
///
/// [1]: https://serde.rs/attributes.html#field-attributes
/// [2]: https://tools.ietf.org/html/rfc3339
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod serde {
    pub use super::datetime::serde::*;
}
