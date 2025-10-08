use core::{
    cmp::Ordering,
    fmt::{self, Debug},
    num::{NonZeroU64, ParseIntError},
    str::FromStr,
};

use macros::model;

#[model(
    proto(raw(protos::ibc::core::client::v1::Height), into, from),
    no_serde
)]
#[derive(Default, Copy)]
#[debug("Height({self})")]
#[derive(Hash)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// TODO: Implement Valuable via Display once https://github.com/tokio-rs/valuable/pull/133 is merged
pub struct Height {
    revision: Option<NonZeroU64>,
    height: u64,
}

// #[cfg(feature = "valuable")]
// impl valuable::Valuable for Height {
//     fn as_value(&self) -> valuable::Value<'_> {
//         valuable::Value::Renderable(valuable::Renderable::Display(self))
//     }

//     fn visit(&self, visit: &mut dyn valuable::Visit) {
//         visit.visit_value(self.as_value());
//     }
// }

impl Height {
    #[must_use]
    pub const fn new(height: u64) -> Self {
        Self {
            revision: None,
            height,
        }
    }

    #[must_use]
    pub const fn new_with_revision(revision: u64, height: u64) -> Self {
        Self {
            revision: NonZeroU64::new(revision),
            height,
        }
    }

    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    #[must_use]
    pub const fn height_mut(&mut self) -> &mut u64 {
        &mut self.height
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        match self.revision {
            Some(revision) => revision.get(),
            None => 0,
        }
    }

    #[must_use]
    pub const fn revision_matches(&self, other: &Self) -> bool {
        match (self.revision, other.revision) {
            (Some(lhs), Some(rhs)) => lhs.get() == rhs.get(),
            _ => true,
        }
    }

    #[must_use]
    pub const fn increment(self) -> Self {
        Self {
            revision: self.revision,
            height: self.height + 1,
        }
    }

    #[must_use]
    pub const fn increment_by(self, height: u64) -> Self {
        Self {
            revision: self.revision,
            height: self.height + height,
        }
    }

    pub fn from_str_allow_zero_revision(s: &str) -> Result<Self, HeightFromStrError> {
        match s.split_once('-') {
            Some((n, h)) => Ok(Self::new_with_revision(n.parse()?, h.parse()?)),
            None => Err(HeightFromStrError::Invalid),
        }
    }
}

// Implement once this is stabilized: https://github.com/rust-lang/rust/issues/42168
// impl Step for Height {
//     fn steps_between(start: &Self, end: &Self) -> Option<usize> {
//         start
//             .revision_matches(end)
//             .then(|| <u64 as Step>::steps_between(&start.height(), &end.height()))
//             .flatten()
//     }

//     fn forward_checked(start: Self, count: usize) -> Option<Self> {
//         start
//             .revision_matches(end)
//             .then(|| <u64 as Step>::forward_checked(&start.height(), &end.height()))
//             .flatten()
//     }

//     fn backward_checked(start: Self, count: usize) -> Option<Self> {
//         start
//             .revision_matches(end)
//             .then(|| <u64 as Step>::backward_checked(&start.height(), &end.height()))
//             .flatten()
//     }
// }

impl FromStr for Height {
    type Err = HeightFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('-') {
            Some((n, h)) => Ok(Self {
                revision: Some(n.parse().map_err(HeightFromStrError::ParseIntError)?),
                height: h.parse().map_err(HeightFromStrError::ParseIntError)?,
            }),
            None => Ok(Self {
                revision: None,
                height: s.parse().map_err(HeightFromStrError::ParseIntError)?,
            }),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum HeightFromStrError {
    #[error("invalid numeric value in height string")]
    ParseIntError(#[from] ParseIntError),
    #[error("invalid height string")]
    Invalid,
}

impl PartialOrd for Height {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Height {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.revision, other.revision) {
            (Some(this_revision), Some(other_revision)) => match this_revision.cmp(&other_revision)
            {
                Ordering::Less => Ordering::Less,
                Ordering::Equal => self.height.cmp(&other.height),
                Ordering::Greater => Ordering::Greater,
            },
            _ => self.height.cmp(&other.height),
        }
    }
}

impl fmt::Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}-{}", self.revision(), self.height)
        } else {
            match self.revision {
                Some(revision_number) => {
                    write!(f, "{}-{}", revision_number, self.height)
                }
                None => {
                    write!(f, "{}", self.height)
                }
            }
        }
    }
}

// #[cfg(feature = "serde")]
impl serde::Serialize for Height {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;

        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            let mut ser = serializer.serialize_tuple(2)?;
            ser.serialize_element(&self.revision())?;
            ser.serialize_element(&self.height())?;
            ser.end()
        }
    }
}

// #[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Height {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            struct HeightStrVisitor;

            impl serde::de::Visitor<'_> for HeightStrVisitor {
                type Value = Height;

                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    formatter.write_str("string representation of Height")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    v.parse().map_err(serde::de::Error::custom)
                }

                fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    v.parse().map_err(serde::de::Error::custom)
                }
            }

            deserializer.deserialize_any(HeightStrVisitor)
        } else {
            <(u64, u64)>::deserialize(deserializer).map(|(n, h)| Height::new_with_revision(n, h))
        }
    }
}

#[cfg(feature = "schemars")]
impl ::schemars::JsonSchema for Height {
    fn schema_name() -> String {
        "Height".to_owned()
    }

    fn schema_id() -> alloc::borrow::Cow<'static, str> {
        alloc::borrow::Cow::Borrowed(concat!(module_path!(), "::Height"))
    }

    fn json_schema(_: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::{
            InstanceType, Metadata, SchemaObject, SingleOrVec, StringValidation,
        };

        SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "A blockchain height, optionally prefixed with a revision \
                    number as per the [IBC Specification]\
                    (https://ibc.cosmos.network/main/ibc/overview/#ibc-client-heights)."
                        .to_owned(),
                ),
                ..Default::default()
            })),
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
            string: Some(Box::new(StringValidation {
                // nonzero revision number - revision height
                pattern: Some(r"([1-9]\d*-)?\d+".to_owned()),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

// #[cfg(feature = "proto")]
pub mod proto {
    use core::num::NonZeroU64;

    use crate::ibc::core::client::height::Height;

    impl From<protos::ibc::core::client::v1::Height> for Height {
        fn from(proto: protos::ibc::core::client::v1::Height) -> Self {
            Self {
                revision: NonZeroU64::new(proto.revision_number),
                height: proto.revision_height,
            }
        }
    }
    impl From<Height> for protos::ibc::core::client::v1::Height {
        fn from(value: Height) -> Self {
            Self {
                revision_number: value.revision.map_or_else(|| 0, NonZeroU64::get),
                revision_height: value.height,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! nz {
        ($n:expr) => {
            const { crate::option_unwrap!(NonZeroU64::new($n)) }
        };
    }

    #[test]
    fn debug() {
        assert_eq!(
            format!(
                "{:?}",
                Height {
                    revision: Some(nz!(1)),
                    height: 1,
                }
            ),
            "Height(1-1)"
        );

        assert_eq!(
            format!(
                "{:?}",
                Height {
                    revision: None,
                    height: 1,
                }
            ),
            "Height(1)"
        );
    }

    #[test]
    fn display() {
        assert_eq!(
            format!(
                "{}",
                Height {
                    revision: Some(nz!(1)),
                    height: 1,
                }
            ),
            "1-1"
        );

        assert_eq!(
            format!(
                "{}",
                Height {
                    revision: None,
                    height: 1,
                }
            ),
            "1"
        );

        assert_eq!(
            format!(
                "{:#}",
                Height {
                    revision: None,
                    height: 1,
                }
            ),
            "0-1"
        );

        assert_eq!(
            format!(
                "{:#}",
                Height {
                    revision: Some(nz!(1)),
                    height: 1,
                }
            ),
            "1-1"
        );
    }

    #[test]
    fn from_str() {
        assert_eq!(
            Height::from_str("1-1"),
            Ok(Height {
                revision: Some(nz!(1)),
                height: 1,
            })
        );

        assert_eq!(
            Height::from_str("1-0"),
            Ok(Height {
                revision: Some(nz!(1)),
                height: 0,
            })
        );

        assert_eq!(
            Height::from_str(&format!("{0}-{0}", u64::MAX)),
            Ok(Height {
                revision: Some(nz!(u64::MAX)),
                height: u64::MAX,
            })
        );

        // will try to parse "2-0" as a u64
        assert!(matches!(
            Height::from_str("4-2-0"),
            Err(HeightFromStrError::ParseIntError(_))
        ));

        assert!(matches!(
            Height::from_str("gibberish"),
            Err(HeightFromStrError::ParseIntError(_))
        ));
    }

    #[test]
    fn cmp() {
        let h1 = Height::new(1);
        let h2 = Height::new(2);

        let h1_1 = Height::new_with_revision(1, 1);
        let h1_2 = Height::new_with_revision(1, 2);

        let h2_1 = Height::new_with_revision(2, 1);
        let h2_2 = Height::new_with_revision(2, 2);

        // no revision
        assert_eq!(h1.cmp(&h1), Ordering::Equal);
        assert_eq!(h2.cmp(&h1), Ordering::Greater);
        assert_eq!(h1.cmp(&h2), Ordering::Less);

        // both same revision
        assert_eq!(h1_1.cmp(&h1_1), Ordering::Equal);
        assert_eq!(h1_2.cmp(&h1_1), Ordering::Greater);
        assert_eq!(h1_1.cmp(&h1_2), Ordering::Less);

        // rhs revision (will be ignored)
        assert_eq!(h1.cmp(&h1_1), Ordering::Equal);
        assert_eq!(h2.cmp(&h1_1), Ordering::Greater);
        assert_eq!(h1.cmp(&h1_2), Ordering::Less);

        // lhs revision (will be ignored)
        assert_eq!(h1_1.cmp(&h1), Ordering::Equal);
        assert_eq!(h1_1.cmp(&h2), Ordering::Less);
        assert_eq!(h1_2.cmp(&h1), Ordering::Greater);

        // both with revision, but different
        assert_eq!(h1_1.cmp(&h2_1), Ordering::Less);
        assert_eq!(h2_1.cmp(&h1_1), Ordering::Greater);

        assert_eq!(h1_2.cmp(&h2_1), Ordering::Less);
        assert_eq!(h2_1.cmp(&h1_2), Ordering::Greater);

        assert_eq!(h1_2.cmp(&h2_2), Ordering::Less);
        assert_eq!(h2_1.cmp(&h1_1), Ordering::Greater);
    }
}
