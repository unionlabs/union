//! The implementation for Version 6 UUIDs.
//!
//! Note that you need to enable the `v6` Cargo feature
//! in order to use this module.

use crate::{Builder, Timestamp, Uuid};

impl Uuid {
    /// Create a new version 6 UUID using the current system time and node ID.
    ///
    /// This method is only available if the `std` feature is enabled.
    ///
    /// This method is a convenient alternative to [`Uuid::new_v6`] that uses the current system time
    /// as the source timestamp.
    ///
    /// Note that usage of this method requires the `v6`, `std`, and `rng` features of this crate
    /// to be enabled.
    #[cfg(all(feature = "std", feature = "rng"))]
    pub fn now_v6(node_id: &[u8; 6]) -> Self {
        let ts = Timestamp::now(crate::timestamp::context::shared_context());

        Self::new_v6(ts, node_id)
    }

    /// Create a new version 6 UUID using the given timestamp and a node ID.
    ///
    /// This is similar to version 1 UUIDs, except that it is lexicographically sortable by timestamp.
    ///
    /// Also see [`Uuid::now_v6`] for a convenient way to generate version 6
    /// UUIDs using the current system time.
    ///
    /// When generating [`Timestamp`]s using a [`ClockSequence`], this function
    /// is only guaranteed to produce unique values if the following conditions
    /// hold:
    ///
    /// 1. The *node ID* is unique for this process,
    /// 2. The *context* is shared across all threads which are generating version 6
    ///    UUIDs,
    /// 3. The [`ClockSequence`] implementation reliably returns unique
    ///    clock sequences (this crate provides [`Context`] for this
    ///    purpose. However you can create your own [`ClockSequence`]
    ///    implementation, if [`Context`] does not meet your needs).
    ///
    /// The NodeID must be exactly 6 bytes long.
    ///
    /// Note that usage of this method requires the `v6` feature of this crate
    /// to be enabled.
    ///
    /// # Examples
    ///
    /// A UUID can be created from a unix [`Timestamp`] with a
    /// [`ClockSequence`]. RFC4122 requires the clock sequence
    /// is seeded with a random value:
    ///
    /// ```rust
    /// # use uuid::{Uuid, Timestamp, Context};
    /// # fn random_seed() -> u16 { 42 }
    /// let context = Context::new(random_seed());
    /// let ts = Timestamp::from_unix(context, 1497624119, 1234);
    ///
    /// let uuid = Uuid::new_v6(ts, &[1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "1e752a1f-3b49-658c-802a-010203040506"
    /// );
    /// ```
    ///
    /// The timestamp can also be created manually as per RFC4122:
    ///
    /// ```
    /// # use uuid::{Uuid, Timestamp, Context, ClockSequence};
    /// # fn random_seed() -> u16 { 42 }
    /// let context = Context::new(random_seed());
    /// let ts = Timestamp::from_rfc4122(14976241191231231313, context.generate_sequence(0, 0) );
    ///
    /// let uuid = Uuid::new_v6(ts, &[1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "fd64c041-1e91-6551-802a-010203040506"
    /// );
    /// ```
    ///
    /// # References
    ///
    /// * [Version 6 UUIDs in Draft RFC: New UUID Formats, Version 4](https://datatracker.ietf.org/doc/html/draft-peabody-dispatch-new-uuid-format-04#section-5.1)
    ///
    /// [`Timestamp`]: v1/struct.Timestamp.html
    /// [`ClockSequence`]: v1/trait.ClockSequence.html
    /// [`Context`]: v1/struct.Context.html
    pub fn new_v6(ts: Timestamp, node_id: &[u8; 6]) -> Self {
        let (ticks, counter) = ts.to_rfc4122();

        Builder::from_sorted_rfc4122_timestamp(ticks, counter, node_id).into_uuid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, Variant, Version};
    use std::string::ToString;

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
        let time: u64 = 1_496_854_535;
        let time_fraction: u32 = 812_946_000;
        let node = [1, 2, 3, 4, 5, 6];
        let context = Context::new(0);

        let uuid = Uuid::new_v6(Timestamp::from_unix(context, time, time_fraction), &node);

        assert_eq!(uuid.get_version(), Some(Version::SortMac));
        assert_eq!(uuid.get_variant(), Variant::RFC4122);
        assert_eq!(
            uuid.hyphenated().to_string(),
            "1e74ba22-0616-6934-8000-010203040506"
        );

        let ts = uuid.get_timestamp().unwrap().to_rfc4122();

        assert_eq!(ts.0 - 0x01B2_1DD2_1381_4000, 14_968_545_358_129_460);

        // Ensure parsing the same UUID produces the same timestamp
        let parsed = Uuid::parse_str("1e74ba22-0616-6934-8000-010203040506").unwrap();

        assert_eq!(
            uuid.get_timestamp().unwrap(),
            parsed.get_timestamp().unwrap()
        );
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
    #[cfg(all(feature = "std", feature = "rng"))]
    fn test_now() {
        let node = [1, 2, 3, 4, 5, 6];

        let uuid = Uuid::now_v6(&node);

        assert_eq!(uuid.get_version(), Some(Version::SortMac));
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
    fn test_new_context() {
        let time: u64 = 1_496_854_535;
        let time_fraction: u32 = 812_946_000;
        let node = [1, 2, 3, 4, 5, 6];

        // This context will wrap
        let context = Context::new((u16::MAX >> 2) - 1);

        let uuid1 = Uuid::new_v6(Timestamp::from_unix(&context, time, time_fraction), &node);

        let time: u64 = 1_496_854_536;

        let uuid2 = Uuid::new_v6(Timestamp::from_unix(&context, time, time_fraction), &node);

        assert_eq!(uuid1.get_timestamp().unwrap().to_rfc4122().1, 16382);
        assert_eq!(uuid2.get_timestamp().unwrap().to_rfc4122().1, 0);

        let time = 1_496_854_535;

        let uuid3 = Uuid::new_v6(Timestamp::from_unix(&context, time, time_fraction), &node);
        let uuid4 = Uuid::new_v6(Timestamp::from_unix(&context, time, time_fraction), &node);

        assert_eq!(uuid3.get_timestamp().unwrap().counter, 1);
        assert_eq!(uuid4.get_timestamp().unwrap().counter, 2);
    }
}
