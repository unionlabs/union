use core::fmt::Debug;

use macros::model;

/// Wrapper type to indicate that a type is to be serialized as an Any.
#[derive(Debug, Clone, PartialEq)]
pub struct Any<T>(pub T);

#[model]
pub struct RawAny {
    pub type_url: String,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub value: Vec<u8>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use frame_support_procedural::DebugNoBound;
    use prost::Message;

    use crate::{
        encoding::{Decode, DecodeErrorOf, Encode, Proto},
        google::protobuf::any::{Any, RawAny},
        TryFromProtoBytesError, TypeUrl,
    };

    impl From<protos::google::protobuf::Any> for RawAny {
        fn from(value: protos::google::protobuf::Any) -> Self {
            Self {
                type_url: value.type_url,
                value: value.value.to_vec(),
            }
        }
    }

    impl From<RawAny> for protos::google::protobuf::Any {
        fn from(value: RawAny) -> Self {
            Self {
                type_url: value.type_url,
                value: value.value.into(),
            }
        }
    }

    impl<T: Encode<Proto> + TypeUrl> From<Any<T>> for protos::google::protobuf::Any {
        fn from(val: Any<T>) -> Self {
            protos::google::protobuf::Any {
                type_url: T::type_url().to_string(),
                value: val.0.encode().into(),
            }
        }
    }

    impl<T: Encode<Proto> + TypeUrl> Encode<Proto> for Any<T> {
        fn encode(self) -> Vec<u8> {
            protos::google::protobuf::Any::from(self).encode_to_vec()
        }
    }
    // NOTE: In order for IntoAny to work, Any cannot implement TypeUrl. If nested Anys are required, you're crazy and I'm not helping you
    // impl TypeUrl for protos::google::protobuf::Any {
    //     const TYPE_URL: &'static str = "/google.protobuf.Any";
    // }

    #[derive(DebugNoBound, thiserror::Error)]
    pub enum TryFromAnyError<T: Decode<Proto, Error: core::error::Error> + TypeUrl> {
        #[error(
        "incorrect type url, expected `{expected}` but found `{found}`",
        expected = T::type_url()
    )]
        IncorrectTypeUrl { found: String },
        #[error("unable to decode inner type")]
        Decode(#[source] DecodeErrorOf<Proto, T>),
    }

    impl<T: Decode<Proto, Error: core::error::Error + PartialEq> + TypeUrl> PartialEq
        for TryFromAnyError<T>
    {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (
                    TryFromAnyError::IncorrectTypeUrl { found: this },
                    TryFromAnyError::IncorrectTypeUrl { found: other },
                ) => this == other,
                (TryFromAnyError::Decode(this), TryFromAnyError::Decode(other)) => this == other,
                _ => false,
            }
        }
    }

    impl<T: Decode<Proto, Error: core::error::Error + Clone> + TypeUrl> Clone for TryFromAnyError<T> {
        fn clone(&self) -> Self {
            match self {
                TryFromAnyError::IncorrectTypeUrl { found } => TryFromAnyError::IncorrectTypeUrl {
                    found: found.clone(),
                },
                TryFromAnyError::Decode(err) => TryFromAnyError::Decode(err.clone()),
            }
        }
    }

    impl<T> TryFrom<protos::google::protobuf::Any> for Any<T>
    where
        T: Decode<Proto, Error: core::error::Error> + TypeUrl,
    {
        type Error = TryFromAnyError<T>;

        fn try_from(value: protos::google::protobuf::Any) -> Result<Self, Self::Error> {
            if value.type_url == T::type_url() {
                T::decode(&value.value)
                    .map_err(TryFromAnyError::Decode)
                    .map(Any)
            } else {
                Err(TryFromAnyError::IncorrectTypeUrl {
                    found: value.type_url,
                })
            }
        }
    }

    impl<T> Decode<Proto> for Any<T>
    where
        T: Decode<Proto, Error: core::error::Error> + TypeUrl,
    {
        type Error = TryFromProtoBytesError<TryFromAnyError<T>>;

        fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
            <protos::google::protobuf::Any as ::prost::Message>::decode(bytes)
                .map_err(TryFromProtoBytesError::Decode)
                .and_then(|proto| {
                    proto
                        .try_into()
                        .map_err(TryFromProtoBytesError::TryFromProto)
                })
        }
    }

    // for use with raw prost generated types
    #[must_use]
    pub fn mk_any<T: prost::Name + prost::Message>(t: &T) -> protos::google::protobuf::Any {
        let bz = t.encode_to_vec();
        protos::google::protobuf::Any {
            type_url: T::type_url(),
            value: bz.into(),
        }
    }
}

/// TODO(unionlabs/union#876): Properly implement google.protobuf.Any json serde
#[cfg(all(feature = "serde", feature = "proto"))] // needs TypeUrl so gate behind proto as well as serde
mod serde {
    use core::marker::PhantomData;

    use serde::{de, ser::SerializeStruct, Deserialize, Serialize};

    use crate::{google::protobuf::any::Any, TypeUrl};

    impl<'de, T> Deserialize<'de> for Any<T>
    where
        T: Deserialize<'de> + crate::TypeUrl,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct AnyVisitor<T>(PhantomData<T>);

            impl<'de, T> serde::de::Visitor<'de> for AnyVisitor<T>
            where
                T: Deserialize<'de> + crate::TypeUrl,
            {
                type Value = Any<T>;

                fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    write!(
                        formatter,
                        "a google.protobuf.Any containing {}",
                        T::type_url()
                    )
                }

                fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    const EXPECTED_MSG: &str = "Object with fields type_url and data";

                    let mut type_url: Option<&str> = None;
                    let mut data: Option<T> = None;

                    for _ in 0..2 {
                        match map
                            .next_key::<&str>()?
                            .ok_or(de::Error::invalid_length(0, &EXPECTED_MSG))?
                        {
                            "type_url" => {
                                let _ = type_url.insert(map.next_value()?);
                            }
                            "data" => {
                                let _ = data.insert(map.next_value()?);
                            }
                            unknown => {
                                return Err(de::Error::unknown_field(
                                    unknown,
                                    &["type_url", "data"],
                                ))
                            }
                        }
                    }

                    match (type_url, data) {
                        (None, None) => Err(de::Error::invalid_length(0, &EXPECTED_MSG)),
                        (None, Some(_)) => Err(de::Error::missing_field("type_url")),
                        (Some(_), None) => Err(de::Error::missing_field("data")),
                        (Some(_), Some(data)) => Ok(Any(data)),
                    }
                }
            }

            deserializer.deserialize_struct(
                "Any",
                &["type_url", "data"],
                AnyVisitor::<T>(PhantomData),
            )
        }
    }

    impl<T> Serialize for Any<T>
    where
        T: Serialize + TypeUrl,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut s = serializer.serialize_struct("Any", 2)?;
            s.serialize_field("type_url", &T::type_url())?;
            s.serialize_field("data", &self.0)?;
            s.end()
        }
    }
}
