#![cfg_attr(docsrs, doc(cfg(feature = "serde")))]

use core::fmt;
use serde::{de, ser};

use super::NaiveDateTime;
use crate::offset::LocalResult;

/// Serialize a `NaiveDateTime` as an RFC 3339 string
///
/// See [the `serde` module](./serde/index.html) for alternate
/// serialization formats.
impl ser::Serialize for NaiveDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        struct FormatWrapped<'a, D: 'a> {
            inner: &'a D,
        }

        impl<'a, D: fmt::Debug> fmt::Display for FormatWrapped<'a, D> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.inner.fmt(f)
            }
        }

        serializer.collect_str(&FormatWrapped { inner: &self })
    }
}

struct NaiveDateTimeVisitor;

impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a formatted date and time string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse().map_err(E::custom)
    }
}

impl<'de> de::Deserialize<'de> for NaiveDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(NaiveDateTimeVisitor)
    }
}

/// Used to serialize/deserialize from nanosecond-precision timestamps
///
/// # Example:
///
/// ```rust
/// # use chrono::{NaiveDate, NaiveDateTime};
/// # use serde_derive::{Deserialize, Serialize};
/// use chrono::naive::serde::ts_nanoseconds;
/// #[derive(Deserialize, Serialize)]
/// struct S {
///     #[serde(with = "ts_nanoseconds")]
///     time: NaiveDateTime
/// }
///
/// let time = NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_nano_opt(02, 04, 59, 918355733).unwrap();
/// let my_s = S {
///     time: time.clone(),
/// };
///
/// let as_string = serde_json::to_string(&my_s)?;
/// assert_eq!(as_string, r#"{"time":1526522699918355733}"#);
/// let my_s: S = serde_json::from_str(&as_string)?;
/// assert_eq!(my_s.time, time);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub mod ts_nanoseconds {
    use core::fmt;
    use serde::{de, ser};

    use super::ne_timestamp;
    use crate::NaiveDateTime;

    /// Serialize a datetime into an integer number of nanoseconds since the epoch
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Serialize;
    /// use chrono::naive::serde::ts_nanoseconds::serialize as to_nano_ts;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_nano_ts")]
    ///     time: NaiveDateTime
    /// }
    ///
    /// let my_s = S {
    ///     time: NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_nano_opt(02, 04, 59, 918355733).unwrap(),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1526522699918355733}"#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(dt.timestamp_nanos())
    }

    /// Deserialize a `NaiveDateTime` from a nanoseconds timestamp
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::NaiveDateTime;
    /// # use serde_derive::Deserialize;
    /// use chrono::naive::serde::ts_nanoseconds::deserialize as from_nano_ts;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_nano_ts")]
    ///     time: NaiveDateTime
    /// }
    ///
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1526522699918355733 }"#)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_i64(NanoSecondsTimestampVisitor)
    }

    pub(super) struct NanoSecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for NanoSecondsTimestampVisitor {
        type Value = NaiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            NaiveDateTime::from_timestamp_opt(value / 1_000_000_000, (value % 1_000_000_000) as u32)
                .ok_or_else(|| E::custom(ne_timestamp(value)))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            NaiveDateTime::from_timestamp_opt(
                value as i64 / 1_000_000_000,
                (value as i64 % 1_000_000_000) as u32,
            )
            .ok_or_else(|| E::custom(ne_timestamp(value)))
        }
    }
}

/// Ser/de to/from optional timestamps in nanoseconds
///
/// Intended for use with `serde`'s `with` attribute.
///
/// # Example:
///
/// ```rust
/// # use chrono::naive::{NaiveDate, NaiveDateTime};
/// # use serde_derive::{Deserialize, Serialize};
/// use chrono::naive::serde::ts_nanoseconds_option;
/// #[derive(Deserialize, Serialize)]
/// struct S {
///     #[serde(with = "ts_nanoseconds_option")]
///     time: Option<NaiveDateTime>
/// }
///
/// let time = Some(NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_nano_opt(02, 04, 59, 918355733).unwrap());
/// let my_s = S {
///     time: time.clone(),
/// };
///
/// let as_string = serde_json::to_string(&my_s)?;
/// assert_eq!(as_string, r#"{"time":1526522699918355733}"#);
/// let my_s: S = serde_json::from_str(&as_string)?;
/// assert_eq!(my_s.time, time);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub mod ts_nanoseconds_option {
    use core::fmt;
    use serde::{de, ser};

    use super::ts_nanoseconds::NanoSecondsTimestampVisitor;
    use crate::NaiveDateTime;

    /// Serialize a datetime into an integer number of nanoseconds since the epoch or none
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::naive::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Serialize;
    /// use chrono::naive::serde::ts_nanoseconds_option::serialize as to_nano_tsopt;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_nano_tsopt")]
    ///     time: Option<NaiveDateTime>
    /// }
    ///
    /// let my_s = S {
    ///     time: Some(NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_nano_opt(02, 04, 59, 918355733).unwrap()),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1526522699918355733}"#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn serialize<S>(opt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *opt {
            Some(ref dt) => serializer.serialize_some(&dt.timestamp_nanos()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize a `NaiveDateTime` from a nanosecond timestamp or none
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::naive::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Deserialize;
    /// use chrono::naive::serde::ts_nanoseconds_option::deserialize as from_nano_tsopt;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_nano_tsopt")]
    ///     time: Option<NaiveDateTime>
    /// }
    ///
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1526522699918355733 }"#)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_option(OptionNanoSecondsTimestampVisitor)
    }

    struct OptionNanoSecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for OptionNanoSecondsTimestampVisitor {
        type Value = Option<NaiveDateTime>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp in nanoseconds or none")
        }

        /// Deserialize a timestamp in nanoseconds since the epoch
        fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            d.deserialize_i64(NanoSecondsTimestampVisitor).map(Some)
        }

        /// Deserialize a timestamp in nanoseconds since the epoch
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        /// Deserialize a timestamp in nanoseconds since the epoch
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }
}

/// Used to serialize/deserialize from microsecond-precision timestamps
///
/// # Example:
///
/// ```rust
/// # use chrono::{NaiveDate, NaiveDateTime};
/// # use serde_derive::{Deserialize, Serialize};
/// use chrono::naive::serde::ts_microseconds;
/// #[derive(Deserialize, Serialize)]
/// struct S {
///     #[serde(with = "ts_microseconds")]
///     time: NaiveDateTime
/// }
///
/// let time = NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_micro_opt(02, 04, 59, 918355).unwrap();
/// let my_s = S {
///     time: time.clone(),
/// };
///
/// let as_string = serde_json::to_string(&my_s)?;
/// assert_eq!(as_string, r#"{"time":1526522699918355}"#);
/// let my_s: S = serde_json::from_str(&as_string)?;
/// assert_eq!(my_s.time, time);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub mod ts_microseconds {
    use core::fmt;
    use serde::{de, ser};

    use super::ne_timestamp;
    use crate::NaiveDateTime;

    /// Serialize a datetime into an integer number of microseconds since the epoch
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Serialize;
    /// use chrono::naive::serde::ts_microseconds::serialize as to_micro_ts;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_micro_ts")]
    ///     time: NaiveDateTime
    /// }
    ///
    /// let my_s = S {
    ///     time: NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_micro_opt(02, 04, 59, 918355).unwrap(),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1526522699918355}"#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(dt.timestamp_micros())
    }

    /// Deserialize a `NaiveDateTime` from a microseconds timestamp
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::NaiveDateTime;
    /// # use serde_derive::Deserialize;
    /// use chrono::naive::serde::ts_microseconds::deserialize as from_micro_ts;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_micro_ts")]
    ///     time: NaiveDateTime
    /// }
    ///
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1526522699918355 }"#)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_i64(MicroSecondsTimestampVisitor)
    }

    pub(super) struct MicroSecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for MicroSecondsTimestampVisitor {
        type Value = NaiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            NaiveDateTime::from_timestamp_opt(
                value / 1_000_000,
                ((value % 1_000_000) * 1000) as u32,
            )
            .ok_or_else(|| E::custom(ne_timestamp(value)))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            NaiveDateTime::from_timestamp_opt(
                (value / 1_000_000) as i64,
                ((value % 1_000_000) * 1_000) as u32,
            )
            .ok_or_else(|| E::custom(ne_timestamp(value)))
        }
    }
}

/// Ser/de to/from optional timestamps in microseconds
///
/// Intended for use with `serde`'s `with` attribute.
///
/// # Example:
///
/// ```rust
/// # use chrono::naive::{NaiveDate, NaiveDateTime};
/// # use serde_derive::{Deserialize, Serialize};
/// use chrono::naive::serde::ts_microseconds_option;
/// #[derive(Deserialize, Serialize)]
/// struct S {
///     #[serde(with = "ts_microseconds_option")]
///     time: Option<NaiveDateTime>
/// }
///
/// let time = Some(NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_micro_opt(02, 04, 59, 918355).unwrap());
/// let my_s = S {
///     time: time.clone(),
/// };
///
/// let as_string = serde_json::to_string(&my_s)?;
/// assert_eq!(as_string, r#"{"time":1526522699918355}"#);
/// let my_s: S = serde_json::from_str(&as_string)?;
/// assert_eq!(my_s.time, time);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub mod ts_microseconds_option {
    use core::fmt;
    use serde::{de, ser};

    use super::ts_microseconds::MicroSecondsTimestampVisitor;
    use crate::NaiveDateTime;

    /// Serialize a datetime into an integer number of microseconds since the epoch or none
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::naive::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Serialize;
    /// use chrono::naive::serde::ts_microseconds_option::serialize as to_micro_tsopt;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_micro_tsopt")]
    ///     time: Option<NaiveDateTime>
    /// }
    ///
    /// let my_s = S {
    ///     time: Some(NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_micro_opt(02, 04, 59, 918355).unwrap()),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1526522699918355}"#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn serialize<S>(opt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *opt {
            Some(ref dt) => serializer.serialize_some(&dt.timestamp_micros()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize a `NaiveDateTime` from a nanosecond timestamp or none
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::naive::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Deserialize;
    /// use chrono::naive::serde::ts_microseconds_option::deserialize as from_micro_tsopt;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_micro_tsopt")]
    ///     time: Option<NaiveDateTime>
    /// }
    ///
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1526522699918355 }"#)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_option(OptionMicroSecondsTimestampVisitor)
    }

    struct OptionMicroSecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for OptionMicroSecondsTimestampVisitor {
        type Value = Option<NaiveDateTime>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp in microseconds or none")
        }

        /// Deserialize a timestamp in microseconds since the epoch
        fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            d.deserialize_i64(MicroSecondsTimestampVisitor).map(Some)
        }

        /// Deserialize a timestamp in microseconds since the epoch
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        /// Deserialize a timestamp in microseconds since the epoch
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }
}

/// Used to serialize/deserialize from millisecond-precision timestamps
///
/// # Example:
///
/// ```rust
/// # use chrono::{NaiveDate, NaiveDateTime};
/// # use serde_derive::{Deserialize, Serialize};
/// use chrono::naive::serde::ts_milliseconds;
/// #[derive(Deserialize, Serialize)]
/// struct S {
///     #[serde(with = "ts_milliseconds")]
///     time: NaiveDateTime
/// }
///
/// let time = NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_milli_opt(02, 04, 59, 918).unwrap();
/// let my_s = S {
///     time: time.clone(),
/// };
///
/// let as_string = serde_json::to_string(&my_s)?;
/// assert_eq!(as_string, r#"{"time":1526522699918}"#);
/// let my_s: S = serde_json::from_str(&as_string)?;
/// assert_eq!(my_s.time, time);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub mod ts_milliseconds {
    use core::fmt;
    use serde::{de, ser};

    use super::ne_timestamp;
    use crate::NaiveDateTime;

    /// Serialize a datetime into an integer number of milliseconds since the epoch
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Serialize;
    /// use chrono::naive::serde::ts_milliseconds::serialize as to_milli_ts;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_milli_ts")]
    ///     time: NaiveDateTime
    /// }
    ///
    /// let my_s = S {
    ///     time: NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_milli_opt(02, 04, 59, 918).unwrap(),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1526522699918}"#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(dt.timestamp_millis())
    }

    /// Deserialize a `NaiveDateTime` from a milliseconds timestamp
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::NaiveDateTime;
    /// # use serde_derive::Deserialize;
    /// use chrono::naive::serde::ts_milliseconds::deserialize as from_milli_ts;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_milli_ts")]
    ///     time: NaiveDateTime
    /// }
    ///
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1526522699918 }"#)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_i64(MilliSecondsTimestampVisitor)
    }

    pub(super) struct MilliSecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for MilliSecondsTimestampVisitor {
        type Value = NaiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            NaiveDateTime::from_timestamp_opt(value / 1000, ((value % 1000) * 1_000_000) as u32)
                .ok_or_else(|| E::custom(ne_timestamp(value)))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            NaiveDateTime::from_timestamp_opt(
                (value / 1000) as i64,
                ((value % 1000) * 1_000_000) as u32,
            )
            .ok_or_else(|| E::custom(ne_timestamp(value)))
        }
    }
}

/// Ser/de to/from optional timestamps in milliseconds
///
/// Intended for use with `serde`'s `with` attribute.
///
/// # Example:
///
/// ```rust
/// # use chrono::naive::{NaiveDate, NaiveDateTime};
/// # use serde_derive::{Deserialize, Serialize};
/// use chrono::naive::serde::ts_milliseconds_option;
/// #[derive(Deserialize, Serialize)]
/// struct S {
///     #[serde(with = "ts_milliseconds_option")]
///     time: Option<NaiveDateTime>
/// }
///
/// let time = Some(NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_milli_opt(02, 04, 59, 918).unwrap());
/// let my_s = S {
///     time: time.clone(),
/// };
///
/// let as_string = serde_json::to_string(&my_s)?;
/// assert_eq!(as_string, r#"{"time":1526522699918}"#);
/// let my_s: S = serde_json::from_str(&as_string)?;
/// assert_eq!(my_s.time, time);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub mod ts_milliseconds_option {
    use core::fmt;
    use serde::{de, ser};

    use super::ts_milliseconds::MilliSecondsTimestampVisitor;
    use crate::NaiveDateTime;

    /// Serialize a datetime into an integer number of milliseconds since the epoch or none
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::naive::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Serialize;
    /// use chrono::naive::serde::ts_milliseconds_option::serialize as to_milli_tsopt;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_milli_tsopt")]
    ///     time: Option<NaiveDateTime>
    /// }
    ///
    /// let my_s = S {
    ///     time: Some(NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_milli_opt(02, 04, 59, 918).unwrap()),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1526522699918}"#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn serialize<S>(opt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *opt {
            Some(ref dt) => serializer.serialize_some(&dt.timestamp_millis()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize a `NaiveDateTime` from a nanosecond timestamp or none
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::naive::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Deserialize;
    /// use chrono::naive::serde::ts_milliseconds_option::deserialize as from_milli_tsopt;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_milli_tsopt")]
    ///     time: Option<NaiveDateTime>
    /// }
    ///
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1526522699918355 }"#)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_option(OptionMilliSecondsTimestampVisitor)
    }

    struct OptionMilliSecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for OptionMilliSecondsTimestampVisitor {
        type Value = Option<NaiveDateTime>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp in milliseconds or none")
        }

        /// Deserialize a timestamp in milliseconds since the epoch
        fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            d.deserialize_i64(MilliSecondsTimestampVisitor).map(Some)
        }

        /// Deserialize a timestamp in milliseconds since the epoch
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        /// Deserialize a timestamp in milliseconds since the epoch
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }
}

/// Used to serialize/deserialize from second-precision timestamps
///
/// # Example:
///
/// ```rust
/// # use chrono::{NaiveDate, NaiveDateTime};
/// # use serde_derive::{Deserialize, Serialize};
/// use chrono::naive::serde::ts_seconds;
/// #[derive(Deserialize, Serialize)]
/// struct S {
///     #[serde(with = "ts_seconds")]
///     time: NaiveDateTime
/// }
///
/// let time = NaiveDate::from_ymd_opt(2015, 5, 15).unwrap().and_hms_opt(10, 0, 0).unwrap();
/// let my_s = S {
///     time: time.clone(),
/// };
///
/// let as_string = serde_json::to_string(&my_s)?;
/// assert_eq!(as_string, r#"{"time":1431684000}"#);
/// let my_s: S = serde_json::from_str(&as_string)?;
/// assert_eq!(my_s.time, time);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub mod ts_seconds {
    use core::fmt;
    use serde::{de, ser};

    use super::ne_timestamp;
    use crate::NaiveDateTime;

    /// Serialize a datetime into an integer number of seconds since the epoch
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Serialize;
    /// use chrono::naive::serde::ts_seconds::serialize as to_ts;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_ts")]
    ///     time: NaiveDateTime
    /// }
    ///
    /// let my_s = S {
    ///     time: NaiveDate::from_ymd_opt(2015, 5, 15).unwrap().and_hms_opt(10, 0, 0).unwrap(),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1431684000}"#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(dt.timestamp())
    }

    /// Deserialize a `NaiveDateTime` from a seconds timestamp
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::NaiveDateTime;
    /// # use serde_derive::Deserialize;
    /// use chrono::naive::serde::ts_seconds::deserialize as from_ts;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_ts")]
    ///     time: NaiveDateTime
    /// }
    ///
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1431684000 }"#)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_i64(SecondsTimestampVisitor)
    }

    pub(super) struct SecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for SecondsTimestampVisitor {
        type Value = NaiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            NaiveDateTime::from_timestamp_opt(value, 0)
                .ok_or_else(|| E::custom(ne_timestamp(value)))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            NaiveDateTime::from_timestamp_opt(value as i64, 0)
                .ok_or_else(|| E::custom(ne_timestamp(value)))
        }
    }
}

/// Ser/de to/from optional timestamps in seconds
///
/// Intended for use with `serde`'s `with` attribute.
///
/// # Example:
///
/// ```rust
/// # use chrono::naive::{NaiveDate, NaiveDateTime};
/// # use serde_derive::{Deserialize, Serialize};
/// use chrono::naive::serde::ts_seconds_option;
/// #[derive(Deserialize, Serialize)]
/// struct S {
///     #[serde(with = "ts_seconds_option")]
///     time: Option<NaiveDateTime>
/// }
///
/// let time = Some(NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_opt(02, 04, 59).unwrap());
/// let my_s = S {
///     time: time.clone(),
/// };
///
/// let as_string = serde_json::to_string(&my_s)?;
/// assert_eq!(as_string, r#"{"time":1526522699}"#);
/// let my_s: S = serde_json::from_str(&as_string)?;
/// assert_eq!(my_s.time, time);
/// # Ok::<(), serde_json::Error>(())
/// ```
pub mod ts_seconds_option {
    use core::fmt;
    use serde::{de, ser};

    use super::ts_seconds::SecondsTimestampVisitor;
    use crate::NaiveDateTime;

    /// Serialize a datetime into an integer number of seconds since the epoch or none
    ///
    /// Intended for use with `serde`s `serialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::naive::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Serialize;
    /// use chrono::naive::serde::ts_seconds_option::serialize as to_tsopt;
    /// #[derive(Serialize)]
    /// struct S {
    ///     #[serde(serialize_with = "to_tsopt")]
    ///     time: Option<NaiveDateTime>
    /// }
    ///
    /// let my_s = S {
    ///     time: Some(NaiveDate::from_ymd_opt(2018, 5, 17).unwrap().and_hms_opt(02, 04, 59).unwrap()),
    /// };
    /// let as_string = serde_json::to_string(&my_s)?;
    /// assert_eq!(as_string, r#"{"time":1526522699}"#);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn serialize<S>(opt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *opt {
            Some(ref dt) => serializer.serialize_some(&dt.timestamp()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize a `NaiveDateTime` from a second timestamp or none
    ///
    /// Intended for use with `serde`s `deserialize_with` attribute.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use chrono::naive::{NaiveDate, NaiveDateTime};
    /// # use serde_derive::Deserialize;
    /// use chrono::naive::serde::ts_seconds_option::deserialize as from_tsopt;
    /// #[derive(Deserialize)]
    /// struct S {
    ///     #[serde(deserialize_with = "from_tsopt")]
    ///     time: Option<NaiveDateTime>
    /// }
    ///
    /// let my_s: S = serde_json::from_str(r#"{ "time": 1431684000 }"#)?;
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn deserialize<'de, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_option(OptionSecondsTimestampVisitor)
    }

    struct OptionSecondsTimestampVisitor;

    impl<'de> de::Visitor<'de> for OptionSecondsTimestampVisitor {
        type Value = Option<NaiveDateTime>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a unix timestamp in seconds or none")
        }

        /// Deserialize a timestamp in seconds since the epoch
        fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            d.deserialize_i64(SecondsTimestampVisitor).map(Some)
        }

        /// Deserialize a timestamp in seconds since the epoch
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        /// Deserialize a timestamp in seconds since the epoch
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }
}

#[test]
fn test_serde_serialize() {
    super::test_encodable_json(serde_json::to_string);
}

#[test]
fn test_serde_deserialize() {
    super::test_decodable_json(|input| serde_json::from_str(input));
}

// Bincode is relevant to test separately from JSON because
// it is not self-describing.
#[test]
fn test_serde_bincode() {
    use crate::NaiveDate;
    use bincode::{deserialize, serialize};

    let dt = NaiveDate::from_ymd_opt(2016, 7, 8).unwrap().and_hms_milli_opt(9, 10, 48, 90).unwrap();
    let encoded = serialize(&dt).unwrap();
    let decoded: NaiveDateTime = deserialize(&encoded).unwrap();
    assert_eq!(dt, decoded);
}

#[test]
fn test_serde_bincode_optional() {
    use crate::prelude::*;
    use crate::serde::ts_nanoseconds_option;
    use bincode::{deserialize, serialize};
    use serde_derive::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Test {
        one: Option<i64>,
        #[serde(with = "ts_nanoseconds_option")]
        two: Option<DateTime<Utc>>,
    }

    let expected =
        Test { one: Some(1), two: Some(Utc.with_ymd_and_hms(1970, 1, 1, 0, 1, 1).unwrap()) };
    let bytes: Vec<u8> = serialize(&expected).unwrap();
    let actual = deserialize::<Test>(&(bytes)).unwrap();

    assert_eq!(expected, actual);
}

// lik? function to convert a LocalResult into a serde-ish Result
pub(crate) fn serde_from<T, E, V>(me: LocalResult<T>, ts: &V) -> Result<T, E>
where
    E: de::Error,
    V: fmt::Display,
    T: fmt::Display,
{
    match me {
        LocalResult::None => Err(E::custom(ne_timestamp(ts))),
        LocalResult::Ambiguous(min, max) => {
            Err(E::custom(SerdeError::Ambiguous { timestamp: ts, min, max }))
        }
        LocalResult::Single(val) => Ok(val),
    }
}

enum SerdeError<V: fmt::Display, D: fmt::Display> {
    NonExistent { timestamp: V },
    Ambiguous { timestamp: V, min: D, max: D },
}

/// Construct a [`SerdeError::NonExistent`]
fn ne_timestamp<T: fmt::Display>(ts: T) -> SerdeError<T, u8> {
    SerdeError::NonExistent::<T, u8> { timestamp: ts }
}

impl<V: fmt::Display, D: fmt::Display> fmt::Debug for SerdeError<V, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ChronoSerdeError({})", self)
    }
}

// impl<V: fmt::Display, D: fmt::Debug> core::error::Error for SerdeError<V, D> {}
impl<V: fmt::Display, D: fmt::Display> fmt::Display for SerdeError<V, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SerdeError::NonExistent { timestamp } => {
                write!(f, "value is not a legal timestamp: {}", timestamp)
            }
            SerdeError::Ambiguous { timestamp, min, max } => write!(
                f,
                "value is an ambiguous timestamp: {}, could be either of {}, {}",
                timestamp, min, max
            ),
        }
    }
}
