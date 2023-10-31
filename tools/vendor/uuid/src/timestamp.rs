//! Generating UUIDs from timestamps.
//!
//! Timestamps are used in a few UUID versions as a source of decentralized
//! uniqueness (as in versions 1 and 6), and as a way to enable sorting (as
//! in versions 6 and 7). Timestamps aren't encoded the same way by all UUID
//! versions so this module provides a single [`Timestamp`] type that can
//! convert between them.
//!
//! # Timestamp representations in UUIDs
//!
//! Versions 1 and 6 UUIDs use a bespoke timestamp that consists of the
//! number of 100ns ticks since `1582-10-15 00:00:00`, along with
//! a counter value to avoid duplicates.
//!
//! Version 7 UUIDs use a more standard timestamp that consists of the
//! number of millisecond ticks since the Unix epoch (`1970-01-01 00:00:00`).
//!
//! # References
//!
//! * [Timestamp in RFC4122](https://www.rfc-editor.org/rfc/rfc4122#section-4.1.4)
//! * [Timestamp in Draft RFC: New UUID Formats, Version 4](https://datatracker.ietf.org/doc/html/draft-peabody-dispatch-new-uuid-format-04#section-6.1)

use crate::Uuid;

/// The number of 100 nanosecond ticks between the RFC4122 epoch
/// (`1582-10-15 00:00:00`) and the Unix epoch (`1970-01-01 00:00:00`).
pub const UUID_TICKS_BETWEEN_EPOCHS: u64 = 0x01B2_1DD2_1381_4000;

/// A timestamp that can be encoded into a UUID.
///
/// This type abstracts the specific encoding, so versions 1, 6, and 7
/// UUIDs can both be supported through the same type, even
/// though they have a different representation of a timestamp.
///
/// # References
///
/// * [Timestamp in RFC4122](https://www.rfc-editor.org/rfc/rfc4122#section-4.1.4)
/// * [Timestamp in Draft RFC: New UUID Formats, Version 4](https://datatracker.ietf.org/doc/html/draft-peabody-dispatch-new-uuid-format-04#section-6.1)
/// * [Clock Sequence in RFC4122](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timestamp {
    pub(crate) seconds: u64,
    pub(crate) nanos: u32,
    #[cfg(any(feature = "v1", feature = "v6"))]
    pub(crate) counter: u16,
}

impl Timestamp {
    /// Get a timestamp representing the current system time.
    ///
    /// This method defers to the standard library's `SystemTime` type.
    ///
    /// # Panics
    ///
    /// This method will panic if calculating the elapsed time since the Unix epoch fails.
    #[cfg(feature = "std")]
    pub fn now(context: impl ClockSequence<Output = u16>) -> Self {
        #[cfg(not(any(feature = "v1", feature = "v6")))]
        {
            let _ = context;
        }

        let (seconds, nanos) = now();

        Timestamp {
            seconds,
            nanos,
            #[cfg(any(feature = "v1", feature = "v6"))]
            counter: context.generate_sequence(seconds, nanos),
        }
    }

    /// Construct a `Timestamp` from an RFC4122 timestamp and counter, as used
    /// in versions 1 and 6 UUIDs.
    ///
    /// # Overflow
    ///
    /// If conversion from RFC4122 ticks to the internal timestamp format would overflow
    /// it will wrap.
    pub const fn from_rfc4122(ticks: u64, counter: u16) -> Self {
        #[cfg(not(any(feature = "v1", feature = "v6")))]
        {
            let _ = counter;
        }

        let (seconds, nanos) = Self::rfc4122_to_unix(ticks);

        Timestamp {
            seconds,
            nanos,
            #[cfg(any(feature = "v1", feature = "v6"))]
            counter,
        }
    }

    /// Construct a `Timestamp` from a Unix timestamp, as used in version 7 UUIDs.
    ///
    /// # Overflow
    ///
    /// If conversion from RFC4122 ticks to the internal timestamp format would overflow
    /// it will wrap.
    pub fn from_unix(context: impl ClockSequence<Output = u16>, seconds: u64, nanos: u32) -> Self {
        #[cfg(not(any(feature = "v1", feature = "v6")))]
        {
            let _ = context;

            Timestamp { seconds, nanos }
        }
        #[cfg(any(feature = "v1", feature = "v6"))]
        {
            let counter = context.generate_sequence(seconds, nanos);

            Timestamp {
                seconds,
                nanos,
                counter,
            }
        }
    }

    /// Get the value of the timestamp as an RFC4122 timestamp and counter,
    /// as used in versions 1 and 6 UUIDs.
    ///
    /// # Overflow
    ///
    /// If conversion from RFC4122 ticks to the internal timestamp format would overflow
    /// it will wrap.
    #[cfg(any(feature = "v1", feature = "v6"))]
    pub const fn to_rfc4122(&self) -> (u64, u16) {
        (
            Self::unix_to_rfc4122_ticks(self.seconds, self.nanos),
            self.counter,
        )
    }

    /// Get the value of the timestamp as a Unix timestamp, as used in version 7 UUIDs.
    ///
    /// # Overflow
    ///
    /// If conversion from RFC4122 ticks to the internal timestamp format would overflow
    /// it will wrap.
    pub const fn to_unix(&self) -> (u64, u32) {
        (self.seconds, self.nanos)
    }

    #[cfg(any(feature = "v1", feature = "v6"))]
    const fn unix_to_rfc4122_ticks(seconds: u64, nanos: u32) -> u64 {
        let ticks = UUID_TICKS_BETWEEN_EPOCHS
            .wrapping_add(seconds.wrapping_mul(10_000_000))
            .wrapping_add(nanos as u64 / 100);

        ticks
    }

    const fn rfc4122_to_unix(ticks: u64) -> (u64, u32) {
        (
            ticks.wrapping_sub(UUID_TICKS_BETWEEN_EPOCHS) / 10_000_000,
            (ticks.wrapping_sub(UUID_TICKS_BETWEEN_EPOCHS) % 10_000_000) as u32 * 100,
        )
    }

    #[deprecated(note = "use `to_unix` instead; this method will be removed in a future release")]
    /// Get the number of fractional nanoseconds in the Unix timestamp.
    ///
    /// This method is deprecated and probably doesn't do what you're expecting it to.
    /// It doesn't return the timestamp as nanoseconds since the Unix epoch, it returns
    /// the fractional seconds of the timestamp.
    pub const fn to_unix_nanos(&self) -> u32 {
        panic!("`Timestamp::to_unix_nanos` is deprecated and will be removed: use `Timestamp::to_unix` instead")
    }
}

pub(crate) const fn encode_rfc4122_timestamp(ticks: u64, counter: u16, node_id: &[u8; 6]) -> Uuid {
    let time_low = (ticks & 0xFFFF_FFFF) as u32;
    let time_mid = ((ticks >> 32) & 0xFFFF) as u16;
    let time_high_and_version = (((ticks >> 48) & 0x0FFF) as u16) | (1 << 12);

    let mut d4 = [0; 8];

    d4[0] = (((counter & 0x3F00) >> 8) as u8) | 0x80;
    d4[1] = (counter & 0xFF) as u8;
    d4[2] = node_id[0];
    d4[3] = node_id[1];
    d4[4] = node_id[2];
    d4[5] = node_id[3];
    d4[6] = node_id[4];
    d4[7] = node_id[5];

    Uuid::from_fields(time_low, time_mid, time_high_and_version, &d4)
}

pub(crate) const fn decode_rfc4122_timestamp(uuid: &Uuid) -> (u64, u16) {
    let bytes = uuid.as_bytes();

    let ticks: u64 = ((bytes[6] & 0x0F) as u64) << 56
        | (bytes[7] as u64) << 48
        | (bytes[4] as u64) << 40
        | (bytes[5] as u64) << 32
        | (bytes[0] as u64) << 24
        | (bytes[1] as u64) << 16
        | (bytes[2] as u64) << 8
        | (bytes[3] as u64);

    let counter: u16 = ((bytes[8] & 0x3F) as u16) << 8 | (bytes[9] as u16);

    (ticks, counter)
}

#[cfg(uuid_unstable)]
pub(crate) const fn encode_sorted_rfc4122_timestamp(
    ticks: u64,
    counter: u16,
    node_id: &[u8; 6],
) -> Uuid {
    let time_high = ((ticks >> 28) & 0xFFFF_FFFF) as u32;
    let time_mid = ((ticks >> 12) & 0xFFFF) as u16;
    let time_low_and_version = ((ticks & 0x0FFF) as u16) | (0x6 << 12);

    let mut d4 = [0; 8];

    d4[0] = (((counter & 0x3F00) >> 8) as u8) | 0x80;
    d4[1] = (counter & 0xFF) as u8;
    d4[2] = node_id[0];
    d4[3] = node_id[1];
    d4[4] = node_id[2];
    d4[5] = node_id[3];
    d4[6] = node_id[4];
    d4[7] = node_id[5];

    Uuid::from_fields(time_high, time_mid, time_low_and_version, &d4)
}

#[cfg(uuid_unstable)]
pub(crate) const fn decode_sorted_rfc4122_timestamp(uuid: &Uuid) -> (u64, u16) {
    let bytes = uuid.as_bytes();

    let ticks: u64 = ((bytes[0]) as u64) << 52
        | (bytes[1] as u64) << 44
        | (bytes[2] as u64) << 36
        | (bytes[3] as u64) << 28
        | (bytes[4] as u64) << 20
        | (bytes[5] as u64) << 12
        | ((bytes[6] & 0xF) as u64) << 8
        | (bytes[7] as u64);

    let counter: u16 = ((bytes[8] & 0x3F) as u16) << 8 | (bytes[9] as u16);

    (ticks, counter)
}

#[cfg(uuid_unstable)]
pub(crate) const fn encode_unix_timestamp_millis(millis: u64, random_bytes: &[u8; 10]) -> Uuid {
    let millis_high = ((millis >> 16) & 0xFFFF_FFFF) as u32;
    let millis_low = (millis & 0xFFFF) as u16;

    let random_and_version =
        (random_bytes[1] as u16 | ((random_bytes[0] as u16) << 8) & 0x0FFF) | (0x7 << 12);

    let mut d4 = [0; 8];

    d4[0] = (random_bytes[2] & 0x3F) | 0x80;
    d4[1] = random_bytes[3];
    d4[2] = random_bytes[4];
    d4[3] = random_bytes[5];
    d4[4] = random_bytes[6];
    d4[5] = random_bytes[7];
    d4[6] = random_bytes[8];
    d4[7] = random_bytes[9];

    Uuid::from_fields(millis_high, millis_low, random_and_version, &d4)
}

#[cfg(uuid_unstable)]
pub(crate) const fn decode_unix_timestamp_millis(uuid: &Uuid) -> u64 {
    let bytes = uuid.as_bytes();

    let millis: u64 = (bytes[0] as u64) << 40
        | (bytes[1] as u64) << 32
        | (bytes[2] as u64) << 24
        | (bytes[3] as u64) << 16
        | (bytes[4] as u64) << 8
        | (bytes[5] as u64);

    millis
}

#[cfg(all(
    feature = "std",
    feature = "js",
    all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown"
    )
))]
fn now() -> (u64, u32) {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        // NOTE: This signature works around https://bugzilla.mozilla.org/show_bug.cgi?id=1787770
        #[wasm_bindgen(js_namespace = Date, catch)]
        fn now() -> Result<f64, JsValue>;
    }

    let now = now().unwrap_throw();

    let secs = (now / 1_000.0) as u64;
    let nanos = ((now % 1_000.0) * 1_000_000.0) as u32;

    (secs, nanos)
}

#[cfg(all(
    feature = "std",
    any(
        not(feature = "js"),
        not(all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown"
        ))
    )
))]
fn now() -> (u64, u32) {
    let dur = std::time::SystemTime::UNIX_EPOCH.elapsed().expect(
        "Getting elapsed time since UNIX_EPOCH. If this fails, we've somehow violated causality",
    );

    (dur.as_secs(), dur.subsec_nanos())
}

/// A counter that can be used by version 1 and version 6 UUIDs to support
/// the uniqueness of timestamps.
///
/// # References
///
/// * [Clock Sequence in RFC4122](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.5)
pub trait ClockSequence {
    /// The type of sequence returned by this counter.
    type Output;

    /// Get the next value in the sequence to feed into a timestamp.
    ///
    /// This method will be called each time a [`Timestamp`] is constructed.
    fn generate_sequence(&self, seconds: u64, subsec_nanos: u32) -> Self::Output;
}

impl<'a, T: ClockSequence + ?Sized> ClockSequence for &'a T {
    type Output = T::Output;
    fn generate_sequence(&self, seconds: u64, subsec_nanos: u32) -> Self::Output {
        (**self).generate_sequence(seconds, subsec_nanos)
    }
}

/// Default implementations for the [`ClockSequence`] trait.
pub mod context {
    use super::ClockSequence;

    #[cfg(any(feature = "v1", feature = "v6"))]
    use atomic::{Atomic, Ordering};

    /// An empty counter that will always return the value `0`.
    ///
    /// This type should be used when constructing timestamps for version 7 UUIDs,
    /// since they don't need a counter for uniqueness.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct NoContext;

    impl ClockSequence for NoContext {
        type Output = u16;

        fn generate_sequence(&self, _seconds: u64, _nanos: u32) -> Self::Output {
            0
        }
    }

    #[cfg(all(any(feature = "v1", feature = "v6"), feature = "std", feature = "rng"))]
    static CONTEXT: Context = Context {
        count: Atomic::new(0),
    };

    #[cfg(all(any(feature = "v1", feature = "v6"), feature = "std", feature = "rng"))]
    static CONTEXT_INITIALIZED: Atomic<bool> = Atomic::new(false);

    #[cfg(all(any(feature = "v1", feature = "v6"), feature = "std", feature = "rng"))]
    pub(crate) fn shared_context() -> &'static Context {
        // If the context is in its initial state then assign it to a random value
        // It doesn't matter if multiple threads observe `false` here and initialize the context
        if CONTEXT_INITIALIZED
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            CONTEXT.count.store(crate::rng::u16(), Ordering::Release);
        }

        &CONTEXT
    }

    /// A thread-safe, wrapping counter that produces 14-bit numbers.
    ///
    /// This type should be used when constructing version 1 and version 6 UUIDs.
    #[derive(Debug)]
    #[cfg(any(feature = "v1", feature = "v6"))]
    pub struct Context {
        count: Atomic<u16>,
    }

    #[cfg(any(feature = "v1", feature = "v6"))]
    impl Context {
        /// Construct a new context that's initialized with the given value.
        ///
        /// The starting value should be a random number, so that UUIDs from
        /// different systems with the same timestamps are less likely to collide.
        /// When the `rng` feature is enabled, prefer the [`Context::new_random`] method.
        pub const fn new(count: u16) -> Self {
            Self {
                count: Atomic::<u16>::new(count),
            }
        }

        /// Construct a new context that's initialized with a random value.
        #[cfg(feature = "rng")]
        pub fn new_random() -> Self {
            Self {
                count: Atomic::<u16>::new(crate::rng::u16()),
            }
        }
    }

    #[cfg(any(feature = "v1", feature = "v6"))]
    impl ClockSequence for Context {
        type Output = u16;

        fn generate_sequence(&self, _seconds: u64, _nanos: u32) -> Self::Output {
            // RFC4122 reserves 2 bits of the clock sequence so the actual
            // maximum value is smaller than `u16::MAX`. Since we unconditionally
            // increment the clock sequence we want to wrap once it becomes larger
            // than what we can represent in a "u14". Otherwise there'd be patches
            // where the clock sequence doesn't change regardless of the timestamp
            self.count.fetch_add(1, Ordering::AcqRel) % (u16::MAX >> 2)
        }
    }
}

#[cfg(all(test, any(feature = "v1", feature = "v6")))]
mod tests {
    use super::*;

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
    fn rfc4122_unix_does_not_panic() {
        // Ensure timestamp conversions never panic
        Timestamp::unix_to_rfc4122_ticks(u64::MAX, 0);
        Timestamp::unix_to_rfc4122_ticks(0, u32::MAX);
        Timestamp::unix_to_rfc4122_ticks(u64::MAX, u32::MAX);

        Timestamp::rfc4122_to_unix(u64::MAX);
    }
}
