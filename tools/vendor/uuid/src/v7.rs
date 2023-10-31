//! The implementation for Version 7 UUIDs.
//!
//! Note that you need to enable the `v7` Cargo feature
//! in order to use this module.

use crate::{rng, std::convert::TryInto, timestamp::Timestamp, Builder, Uuid};

impl Uuid {
    /// Create a new version 7 UUID using the current time value and random bytes.
    ///
    /// This method is a convenient alternative to [`Uuid::new_v7`] that uses the current system time
    /// as the source timestamp.
    #[cfg(feature = "std")]
    pub fn now_v7() -> Self {
        Self::new_v7(Timestamp::now(crate::NoContext))
    }

    /// Create a new version 7 UUID using a time value and random bytes.
    ///
    /// When the `std` feature is enabled, you can also use [`Uuid::now_v7`].
    ///
    /// Note that usage of this method requires the `v7` feature of this crate
    /// to be enabled.
    ///
    /// Also see [`Uuid::now_v7`] for a convenient way to generate version 7
    /// UUIDs using the current system time.
    ///
    /// # Examples
    ///
    /// A v7 UUID can be created from a unix [`Timestamp`] plus a 128 bit
    /// random number. When supplied as such, the data will be
    ///
    /// ```rust
    /// # use uuid::{Uuid, Timestamp, NoContext};
    /// let ts = Timestamp::from_unix(NoContext, 1497624119, 1234);
    ///
    /// let uuid = Uuid::new_v7(ts);
    ///
    /// assert!(
    ///     uuid.hyphenated().to_string().starts_with("015cb15a-86d8-7")
    /// );
    /// ```
    ///
    /// # References
    ///
    /// * [Version 7 UUIDs in Draft RFC: New UUID Formats, Version 4](https://datatracker.ietf.org/doc/html/draft-peabody-dispatch-new-uuid-format-04#section-5.2)
    pub fn new_v7(ts: Timestamp) -> Self {
        let (secs, nanos) = ts.to_unix();
        let millis = (secs * 1000).saturating_add(nanos as u64 / 1_000_000);

        Builder::from_unix_timestamp_millis(millis, &rng::bytes()[..10].try_into().unwrap())
            .into_uuid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{std::string::ToString, NoContext, Variant, Version};
    #[cfg(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown"
    ))]
    use wasm_bindgen_test::*;

    #[test]
    #[cfg_attr(
        all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown"
        ),
        wasm_bindgen_test
    )]
    fn test_new() {
        let ts: u64 = 1645557742000;

        let seconds = ts / 1000;
        let nanos = ((ts % 1000) * 1_000_000) as u32;

        let uuid = Uuid::new_v7(Timestamp::from_unix(NoContext, seconds, nanos));
        let uustr = uuid.hyphenated().to_string();

        assert_eq!(uuid.get_version(), Some(Version::SortRand));
        assert_eq!(uuid.get_variant(), Variant::RFC4122);
        assert!(uuid.hyphenated().to_string().starts_with("017f22e2-79b0-7"));

        // Ensure parsing the same UUID produces the same timestamp
        let parsed = Uuid::parse_str(uustr.as_str()).unwrap();

        assert_eq!(uuid, parsed);
    }

    #[test]
    #[cfg_attr(
        all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown"
        ),
        wasm_bindgen_test
    )]
    #[cfg(feature = "std")]
    fn test_now() {
        let uuid = Uuid::now_v7();

        assert_eq!(uuid.get_version(), Some(Version::SortRand));
        assert_eq!(uuid.get_variant(), Variant::RFC4122);
    }

    #[test]
    #[cfg_attr(
        all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown"
        ),
        wasm_bindgen_test
    )]
    fn test_sorting() {
        let time1: u64 = 1_496_854_535;
        let time_fraction1: u32 = 812_000_000;

        let time2 = time1 + 4000;
        let time_fraction2 = time_fraction1;

        let uuid1 = Uuid::new_v7(Timestamp::from_unix(NoContext, time1, time_fraction1));
        let uuid2 = Uuid::new_v7(Timestamp::from_unix(NoContext, time2, time_fraction2));

        assert!(uuid1.as_bytes() < uuid2.as_bytes());
        assert!(uuid1.to_string() < uuid2.to_string());
    }

    #[test]
    #[cfg_attr(
        all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown"
        ),
        wasm_bindgen_test
    )]
    fn test_new_timestamp_roundtrip() {
        let time: u64 = 1_496_854_535;
        let time_fraction: u32 = 812_000_000;

        let ts = Timestamp::from_unix(NoContext, time, time_fraction);

        let uuid = Uuid::new_v7(ts);

        let decoded_ts = uuid.get_timestamp().unwrap();

        assert_eq!(ts.to_unix(), decoded_ts.to_unix());
    }
}
