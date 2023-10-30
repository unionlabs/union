//! The implementation for Version 1 UUIDs.
//!
//! This module is soft-deprecated. Instead of using the `Context` type re-exported here,
//! use the one from the crate root.

use crate::{Builder, Uuid};

#[deprecated(note = "use types from the crate root instead")]
pub use crate::{timestamp::context::Context, Timestamp};

impl Uuid {
    /// Create a new version 1 UUID using the current system time and node ID.
    ///
    /// This method is only available if both the `std` and `rng` features are enabled.
    ///
    /// This method is a convenient alternative to [`Uuid::new_v1`] that uses the current system time
    /// as the source timestamp.
    ///
    /// Note that usage of this method requires the `v1`, `std`, and `rng` features of this crate
    /// to be enabled.
    #[cfg(all(feature = "std", feature = "rng"))]
    pub fn now_v1(node_id: &[u8; 6]) -> Self {
        let ts = Timestamp::now(crate::timestamp::context::shared_context());

        Self::new_v1(ts, node_id)
    }

    /// Create a new version 1 UUID using the given timestamp and node ID.
    ///
    /// Also see [`Uuid::now_v1`] for a convenient way to generate version 1
    /// UUIDs using the current system time.
    ///
    /// When generating [`Timestamp`]s using a [`ClockSequence`], this function
    /// is only guaranteed to produce unique values if the following conditions
    /// hold:
    ///
    /// 1. The *node ID* is unique for this process,
    /// 2. The *context* is shared across all threads which are generating version 1
    ///    UUIDs,
    /// 3. The [`ClockSequence`] implementation reliably returns unique
    ///    clock sequences (this crate provides [`Context`] for this
    ///    purpose. However you can create your own [`ClockSequence`]
    ///    implementation, if [`Context`] does not meet your needs).
    ///
    /// Note that usage of this method requires the `v1` feature of this crate
    /// to be enabled.
    ///
    /// # Examples
    ///
    /// A UUID can be created from a unix [`Timestamp`] with a
    /// [`ClockSequence`]. RFC4122 requires the clock sequence
    /// is seeded with a random value:
    ///
    /// ```
    /// # use uuid::{Timestamp, Context};
    /// # use uuid::Uuid;
    /// # fn random_seed() -> u16 { 42 }
    /// let context = Context::new(random_seed());
    /// let ts = Timestamp::from_unix(&context, 1497624119, 1234);
    ///
    /// let uuid = Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "f3b4958c-52a1-11e7-802a-010203040506"
    /// );
    /// ```
    ///
    /// The timestamp can also be created manually as per RFC4122:
    ///
    /// ```
    /// # use uuid::{Uuid, Timestamp, Context, ClockSequence};
    /// let context = Context::new(42);
    /// let ts = Timestamp::from_rfc4122(14976234442241191232, context.generate_sequence(0, 0));
    ///
    /// let uuid = Uuid::new_v1(ts, &[1, 2, 3, 4, 5, 6]);
    ///
    /// assert_eq!(
    ///     uuid.hyphenated().to_string(),
    ///     "b2c1ad40-45e0-1fd6-802a-010203040506"
    /// );
    /// ```
    ///
    /// # References
    ///
    /// * [Version 1 UUIDs in RFC4122](https://www.rfc-editor.org/rfc/rfc4122#section-4.2)
    ///
    /// [`Timestamp`]: v1/struct.Timestamp.html
    /// [`ClockSequence`]: v1/trait.ClockSequence.html
    /// [`Context`]: v1/struct.Context.html
    pub fn new_v1(ts: Timestamp, node_id: &[u8; 6]) -> Self {
        let (ticks, counter) = ts.to_rfc4122();

        Builder::from_rfc4122_timestamp(ticks, counter, node_id).into_uuid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{std::string::ToString, Variant, Version};
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

        let uuid = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);

        assert_eq!(uuid.get_version(), Some(Version::Mac));
        assert_eq!(uuid.get_variant(), Variant::RFC4122);
        assert_eq!(
            uuid.hyphenated().to_string(),
            "20616934-4ba2-11e7-8000-010203040506"
        );

        let ts = uuid.get_timestamp().unwrap().to_rfc4122();

        assert_eq!(ts.0 - 0x01B2_1DD2_1381_4000, 14_968_545_358_129_460);

        // Ensure parsing the same UUID produces the same timestamp
        let parsed = Uuid::parse_str("20616934-4ba2-11e7-8000-010203040506").unwrap();

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

        let uuid = Uuid::now_v1(&node);

        assert_eq!(uuid.get_version(), Some(Version::Mac));
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

        let uuid1 = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);

        let time: u64 = 1_496_854_536;

        let uuid2 = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);

        assert_eq!(uuid1.get_timestamp().unwrap().to_rfc4122().1, 16382);
        assert_eq!(uuid2.get_timestamp().unwrap().to_rfc4122().1, 0);

        let time = 1_496_854_535;

        let uuid3 = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);
        let uuid4 = Uuid::new_v1(Timestamp::from_unix(&context, time, time_fraction), &node);

        assert_eq!(uuid3.get_timestamp().unwrap().to_rfc4122().1, 1);
        assert_eq!(uuid4.get_timestamp().unwrap().to_rfc4122().1, 2);
    }
}
