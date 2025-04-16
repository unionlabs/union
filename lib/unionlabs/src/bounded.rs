macro_rules! bounded_int {
    ($(
        $(#[non_zero($NonZero:ty)])?
        pub $Struct:ident($ty:ty);
    )+) => {
        $(
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
            pub struct $Struct<const MIN: $ty, const MAX: $ty = { <$ty>::MAX }>($ty);

            impl<const MIN: $ty, const MAX: $ty> $Struct<MIN, MAX> {
                #[must_use]
                pub const fn inner(self) -> $ty {
                    self.0
                }

                #[must_use]
                pub fn add(&self, other: &$ty) -> Self {
                    Self::new(self.inner() + other).expect("arithmetic overflow")
                }
            }

            impl<const MIN: $ty, const MAX: $ty> core::fmt::Debug for $Struct<MIN, MAX> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(
                        f,
                        "{}<{min}..={max}>({})",
                        stringify!($Struct),
                        self.0,
                        min = if MIN == <$ty>::MIN { "MIN".to_owned() } else { format!("{MIN}") },
                        max = if MAX == <$ty>::MAX { "MAX".to_owned() } else { format!("{MAX}") },
                    )
                }
            }

            #[cfg(feature = "bincode")]
            impl<const MIN: $ty, const MAX: $ty> bincode::Encode for $Struct<MIN, MAX> {
                fn encode<E: bincode::enc::Encoder>(
                    &self,
                    encoder: &mut E,
                ) -> Result<(), bincode::error::EncodeError> {
                    self.0.encode(encoder)
                }
            }

            #[cfg(feature = "bincode")]
            impl<Context, const MIN: $ty, const MAX: $ty> bincode::Decode<Context> for $Struct<MIN, MAX> {
                fn decode<D: bincode::de::Decoder<Context = Context>>(
                    decoder: &mut D,
                ) -> Result<Self, bincode::error::DecodeError> {
                    Self::new(<$ty as bincode::Decode<Context>>::decode(decoder)?)
                        .map_err(|err| bincode::error::DecodeError::OtherString(err.to_string()))
                }
            }

            #[cfg(feature = "bincode")]
            impl<'de, Context, const MIN: $ty, const MAX: $ty> bincode::BorrowDecode<'de, Context> for $Struct<MIN, MAX> {
                fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = Context>>(
                    decoder: &mut D,
                ) -> Result<Self, bincode::error::DecodeError> {
                    bincode::Decode::decode(decoder)
                }
            }

            impl<const MIN: $ty, const MAX: $ty> serde::Serialize for $Struct<MIN, MAX> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.0.serialize(serializer)
                }
            }

            impl<'de, const MIN: $ty, const MAX: $ty> serde::Deserialize<'de> for $Struct<MIN, MAX> {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    <$ty>::deserialize(deserializer)
                        .map(Self::new)?
                        .map_err(|err| {
                            serde::de::Error::invalid_value(
                                serde::de::Unexpected::Other(&format!("{}_{}", err.found, stringify!($ty))),
                                &format!("a `{}` between {}..={}", stringify!($ty), err.min, err.max).as_str()
                                    as &dyn serde::de::Expected,
                            )
                        })
                }
            }

            impl<const MIN: $ty, const MAX: $ty> TryFrom<$ty> for $Struct<MIN, MAX> {
                type Error = BoundedIntError<$ty>;

                fn try_from(n: $ty) -> Result<Self, Self::Error> {
                    Self::new(n)
                }
            }

            impl<const MIN: $ty, const MAX: $ty> From<$Struct<MIN, MAX>> for $ty {
                fn from(value: $Struct<MIN, MAX>) -> Self {
                    value.0
                }
            }

            impl<const MIN: $ty, const MAX: $ty> $Struct<MIN, MAX> {
                pub const fn new_const(n: $ty) -> Result<Self, BoundedIntError<$ty>> {
                    const { assert!(MIN < MAX) };

                    if n >= MIN && n <= MAX {
                        Ok(Self(n))
                    } else {
                        Err(BoundedIntError {
                            max: MAX,
                            min: MIN,
                            found: n,
                        })
                    }
                }

                pub fn new<T: Copy + TryInto<$ty>>(n: T) -> Result<Self, BoundedIntError<$ty, T>> {
                    const { assert!(MIN < MAX) };

                    let m = n.try_into().map_err(|_| BoundedIntError {
                        max: MAX,
                        min: MIN,
                        found: n,
                    })?;

                    if m >= MIN && m <= MAX {
                        Ok(Self(m))
                    } else {
                        Err(BoundedIntError {
                            max: MAX,
                            min: MIN,
                            found: n,
                        })
                    }
                }
            }

            impl<const MIN: $ty, const MAX: $ty> core::str::FromStr for $Struct<MIN, MAX> {
                type Err = BoundedIntParseError<$ty>;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    s.parse::<$ty>()
                        .map_err(BoundedIntParseError::Parse)
                        .and_then(|n| n.try_into().map_err(BoundedIntParseError::Value))
                }
            }

            impl<const MIN: $ty, const MAX: $ty> core::fmt::Display for $Struct<MIN, MAX> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{}", self.0)
                }
            }

            $(
                const _: () = assert!(
                    <$ty>::MIN.abs_diff(0) == 0,
                    concat!(
                        "Extra assertion that [`",
                        stringify!($NonZero),
                        "`]",
                        " is the same as [`",
                        stringify!($Struct),
                        "<1, ",
                        stringify!($ty),
                        ">`]."
                    ),
                );

                const _: $ty = match <$NonZero>::new(1) {
                    Some(n) => n.get(),
                    None => unreachable!(),
                };

                impl From<$NonZero> for $Struct<1, { <$ty>::MAX }> {
                    fn from(value: $NonZero) -> Self {
                        Self(value.get())
                    }
                }

                impl From<$Struct<1, { <$ty>::MAX }>> for $NonZero {
                    fn from(value: $Struct<1, { <$ty>::MAX }>) -> Self {
                        Self::new(value.inner()).expect("value is > 0 as per const bounds; qed;")
                    }
                }
            )?
        )+
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("expected a value between {min}..={max}, found {found}")]
pub struct BoundedIntError<T, U = T> {
    min: T,
    max: T,
    found: U,
}

impl<T, U> BoundedIntError<T, U> {
    pub fn min(&self) -> &T {
        &self.min
    }

    pub fn max(&self) -> &T {
        &self.max
    }

    pub fn found(&self) -> &U {
        &self.found
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum BoundedIntParseError<T> {
    #[error(transparent)]
    Parse(core::num::ParseIntError),
    #[error(transparent)]
    Value(BoundedIntError<T>),
}

bounded_int! {
    pub BoundedI8(i8);
    pub BoundedI16(i16);
    pub BoundedI32(i32);
    pub BoundedI64(i64);
    pub BoundedI128(i128);

    #[non_zero(core::num::NonZeroU8)]
    pub BoundedU8(u8);
    #[non_zero(core::num::NonZeroU16)]
    pub BoundedU16(u16);
    #[non_zero(core::num::NonZeroU32)]
    pub BoundedU32(u32);
    #[non_zero(core::num::NonZeroU64)]
    pub BoundedU64(u64);
    #[non_zero(core::num::NonZeroU128)]
    pub BoundedU128(u128);

    pub BoundedIsize(isize);
    #[non_zero(core::num::NonZeroUsize)]
    pub BoundedUsize(usize);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::test_utils::assert_json_roundtrip;

    #[test]
    fn serde() {
        assert_eq!(
            serde_json::from_value::<BoundedI8<0, 10>>(json! { 1 }).unwrap(),
            BoundedI8::<0, 10>::new(1).unwrap(),
        );

        assert_eq!(
            serde_json::from_value::<BoundedI8<0, 10>>(json! { -1 })
                .unwrap_err()
                .to_string(),
            "invalid value: -1_i8, expected a `i8` between 0..=10",
        );

        assert_json_roundtrip(&BoundedU32::<10, 100>::new(45).unwrap());
    }

    #[test]
    fn serde_roundtrip() {
        assert_json_roundtrip(&BoundedU32::<10, 100>::new(45).unwrap());
        assert_json_roundtrip(&BoundedI64::<-10, 1337>::new(45).unwrap());
    }

    #[test]
    fn new() {
        const I8_MIN: i16 = i8::MIN as i16;
        const I8_MAX: i16 = i8::MAX as i16;

        type WeirdI8 = BoundedI16<I8_MIN, I8_MAX>;

        assert_eq!(WeirdI8::new(I8_MIN).unwrap().inner(), I8_MIN);

        assert_eq!(WeirdI8::new(I8_MAX).unwrap().inner(), I8_MAX);

        assert_eq!(
            WeirdI8::new(I8_MIN - 1),
            Err(BoundedIntError {
                min: I8_MIN,
                max: I8_MAX,
                found: I8_MIN - 1,
            })
        );

        assert_eq!(
            WeirdI8::new(I8_MAX + 1),
            Err(BoundedIntError {
                min: I8_MIN,
                max: I8_MAX,
                found: I8_MAX + 1,
            })
        );
    }
}
