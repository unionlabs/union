pub mod protobuf {
    use crate::include_proto;
    include_proto!("google.protobuf.rs");

    // source: https://github.com/tokio-rs/prost/blob/master/prost-types/src/lib.rs
    use core::convert::TryFrom;
    use core::i32;
    use core::i64;
    use core::time;

    // The Protobuf `Duration` and `Timestamp` types can't delegate to the standard library equivalents
    // because the Protobuf versions are signed. To make them easier to work with, `From` conversions
    // are defined in both directions.

    const NANOS_PER_SECOND: i32 = 1_000_000_000;
    const NANOS_MAX: i32 = NANOS_PER_SECOND - 1;

    impl Duration {
        /// Normalizes the duration to a canonical format.
        pub fn normalize(&mut self) {
            // Make sure nanos is in the range.
            if self.nanos <= -NANOS_PER_SECOND || self.nanos >= NANOS_PER_SECOND {
                if let Some(seconds) = self
                    .seconds
                    .checked_add((self.nanos / NANOS_PER_SECOND) as i64)
                {
                    self.seconds = seconds;
                    self.nanos %= NANOS_PER_SECOND;
                } else if self.nanos < 0 {
                    // Negative overflow! Set to the least normal value.
                    self.seconds = i64::MIN;
                    self.nanos = -NANOS_MAX;
                } else {
                    // Positive overflow! Set to the greatest normal value.
                    self.seconds = i64::MAX;
                    self.nanos = NANOS_MAX;
                }
            }

            // nanos should have the same sign as seconds.
            if self.seconds < 0 && self.nanos > 0 {
                if let Some(seconds) = self.seconds.checked_add(1) {
                    self.seconds = seconds;
                    self.nanos -= NANOS_PER_SECOND;
                } else {
                    // Positive overflow! Set to the greatest normal value.
                    debug_assert_eq!(self.seconds, i64::MAX);
                    self.nanos = NANOS_MAX;
                }
            } else if self.seconds > 0 && self.nanos < 0 {
                if let Some(seconds) = self.seconds.checked_sub(1) {
                    self.seconds = seconds;
                    self.nanos += NANOS_PER_SECOND;
                } else {
                    // Negative overflow! Set to the least normal value.
                    debug_assert_eq!(self.seconds, i64::MIN);
                    self.nanos = -NANOS_MAX;
                }
            }
        }
    }

    /// Converts a `std::time::Duration` to a `Duration`.
    impl From<time::Duration> for Duration {
        fn from(duration: time::Duration) -> Duration {
            let seconds = duration.as_secs();
            let seconds = if seconds > i64::MAX as u64 {
                i64::MAX
            } else {
                seconds as i64
            };
            let nanos = duration.subsec_nanos();
            let nanos = if nanos > i32::MAX as u32 {
                i32::MAX
            } else {
                nanos as i32
            };
            let mut duration = Duration { seconds, nanos };
            duration.normalize();
            duration
        }
    }

    impl TryFrom<Duration> for time::Duration {
        type Error = time::Duration;

        /// Converts a `Duration` to a result containing a positive (`Ok`) or negative (`Err`)
        /// `std::time::Duration`.
        fn try_from(mut duration: Duration) -> Result<time::Duration, time::Duration> {
            duration.normalize();
            if duration.seconds >= 0 {
                Ok(time::Duration::new(
                    duration.seconds as u64,
                    duration.nanos as u32,
                ))
            } else {
                Err(time::Duration::new(
                    (-duration.seconds) as u64,
                    (-duration.nanos) as u32,
                ))
            }
        }
    }

    impl Timestamp {
        /// Normalizes the timestamp to a canonical format.
        #[cfg(feature = "std")]
        pub fn normalize(&mut self) {
            // Make sure nanos is in the range.
            if self.nanos <= -NANOS_PER_SECOND || self.nanos >= NANOS_PER_SECOND {
                if let Some(seconds) = self
                    .seconds
                    .checked_add((self.nanos / NANOS_PER_SECOND) as i64)
                {
                    self.seconds = seconds;
                    self.nanos %= NANOS_PER_SECOND;
                } else if self.nanos < 0 {
                    // Negative overflow! Set to the earliest normal value.
                    self.seconds = i64::MIN;
                    self.nanos = 0;
                } else {
                    // Positive overflow! Set to the latest normal value.
                    self.seconds = i64::MAX;
                    self.nanos = 999_999_999;
                }
            }

            // For Timestamp nanos should be in the range [0, 999999999].
            if self.nanos < 0 {
                if let Some(seconds) = self.seconds.checked_sub(1) {
                    self.seconds = seconds;
                    self.nanos += NANOS_PER_SECOND;
                } else {
                    // Negative overflow! Set to the earliest normal value.
                    debug_assert_eq!(self.seconds, i64::MIN);
                    self.nanos = 0;
                }
            }
        }
    }

    /// Implements the unstable/naive version of `Eq`: a basic equality check on the internal fields of the `Timestamp`.
    /// This implies that `normalized_ts != non_normalized_ts` even if `normalized_ts == non_normalized_ts.normalized()`.
    #[cfg(feature = "std")]
    impl Eq for Timestamp {}

    #[cfg(feature = "std")]
    #[allow(clippy::derived_hash_with_manual_eq)] // Derived logic is correct: comparing the 2 fields for equality
    impl std::hash::Hash for Timestamp {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.seconds.hash(state);
            self.nanos.hash(state);
        }
    }

    #[cfg(feature = "std")]
    impl From<std::time::SystemTime> for Timestamp {
        fn from(system_time: std::time::SystemTime) -> Timestamp {
            let (seconds, nanos) = match system_time.duration_since(std::time::UNIX_EPOCH) {
                Ok(duration) => {
                    let seconds = i64::try_from(duration.as_secs()).unwrap();
                    (seconds, duration.subsec_nanos() as i32)
                }
                Err(error) => {
                    let duration = error.duration();
                    let seconds = i64::try_from(duration.as_secs()).unwrap();
                    let nanos = duration.subsec_nanos() as i32;
                    if nanos == 0 {
                        (-seconds, 0)
                    } else {
                        (-seconds - 1, 1_000_000_000 - nanos)
                    }
                }
            };
            Timestamp { seconds, nanos }
        }
    }

    /// Indicates that a [`Timestamp`] could not be converted to
    /// [`SystemTime`][std::time::SystemTime] because it is out of range.
    ///
    /// The range of times that can be represented by `SystemTime` depends on the platform.
    /// All `Timestamp`s are likely representable on 64-bit Unix-like platforms, but
    /// other platforms, such as Windows and 32-bit Linux, may not be able to represent
    /// the full range of `Timestamp`s.
    #[cfg(feature = "std")]
    #[derive(Debug)]
    #[non_exhaustive]
    pub struct TimestampOutOfSystemRangeError {
        pub timestamp: Timestamp,
    }

    #[cfg(feature = "std")]
    impl core::fmt::Display for TimestampOutOfSystemRangeError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(
                f,
                "{self:?} is not representable as a `SystemTime` because it is out of range"
            )
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for TimestampOutOfSystemRangeError {}

    #[cfg(feature = "std")]
    impl TryFrom<Timestamp> for std::time::SystemTime {
        type Error = TimestampOutOfSystemRangeError;

        fn try_from(mut timestamp: Timestamp) -> Result<std::time::SystemTime, Self::Error> {
            let orig_timestamp = timestamp.clone();
            timestamp.normalize();

            let system_time = if timestamp.seconds >= 0 {
                std::time::UNIX_EPOCH
                    .checked_add(time::Duration::from_secs(timestamp.seconds as u64))
            } else {
                std::time::UNIX_EPOCH
                    .checked_sub(time::Duration::from_secs((-timestamp.seconds) as u64))
            };

            let system_time = system_time.and_then(|system_time| {
                system_time.checked_add(time::Duration::from_nanos(timestamp.nanos as u64))
            });

            system_time.ok_or(TimestampOutOfSystemRangeError {
                timestamp: orig_timestamp,
            })
        }
    }

    #[cfg(any(feature = "borsh", feature = "parity-scale-codec"))]
    mod sealed {
        use super::Any;

        use alloc::string::String;
        use alloc::vec::Vec;

        #[cfg_attr(
            feature = "parity-scale-codec",
            derive(
                parity_scale_codec::Encode,
                parity_scale_codec::Decode,
                scale_info::TypeInfo
            )
        )]
        #[cfg_attr(
            feature = "borsh",
            derive(borsh::BorshSerialize, borsh::BorshDeserialize)
        )]
        struct InnerAny {
            pub type_url: String,
            pub value: Vec<u8>,
        }

        #[cfg(feature = "borsh")]
        impl borsh::BorshSerialize for Any {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> borsh::maybestd::io::Result<()> {
                let inner_any = InnerAny {
                    type_url: self.type_url.clone(),
                    value: self.value.clone(),
                };

                borsh::BorshSerialize::serialize(&inner_any, writer)
            }
        }

        #[cfg(feature = "borsh")]
        impl borsh::BorshDeserialize for Any {
            fn deserialize_reader<R: borsh::maybestd::io::Read>(
                reader: &mut R,
            ) -> borsh::maybestd::io::Result<Self> {
                let inner_any = InnerAny::deserialize_reader(reader)?;

                Ok(Any {
                    type_url: inner_any.type_url,
                    value: inner_any.value,
                })
            }
        }

        #[cfg(feature = "parity-scale-codec")]
        impl parity_scale_codec::Encode for Any {
            fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, writer: &mut T) {
                let inner_any = InnerAny {
                    type_url: self.type_url.clone(),
                    value: self.value.clone(),
                };
                inner_any.encode_to(writer);
            }
        }
        #[cfg(feature = "parity-scale-codec")]
        impl parity_scale_codec::Decode for Any {
            fn decode<I: parity_scale_codec::Input>(
                input: &mut I,
            ) -> Result<Self, parity_scale_codec::Error> {
                let inner_any = InnerAny::decode(input)?;
                Ok(Any {
                    type_url: inner_any.type_url.clone(),
                    value: inner_any.value,
                })
            }
        }

        #[cfg(feature = "parity-scale-codec")]
        impl scale_info::TypeInfo for Any {
            type Identity = Self;

            fn type_info() -> scale_info::Type {
                scale_info::Type::builder()
                    .path(scale_info::Path::new("Any", "ibc_proto::google::protobuf"))
                    // i128 is chosen before we represent the timestamp is nanoseconds, which is represented as a i128 by Time
                    .composite(
                        scale_info::build::Fields::named()
                            .field(|f| f.ty::<String>().name("type_url").type_name("String"))
                            .field(|f| f.ty::<Vec<u8>>().name("value").type_name("Vec<u8>")),
                    )
            }
        }
    }
}
