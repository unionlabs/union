use core::fmt::Debug;

use frame_support_procedural::DebugNoBound;
use macros::model;
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    encoding::{Decode, DecodeErrorOf, Encode, Proto},
    TryFromProtoBytesError, TypeUrl,
};

/// Wrapper type to indicate that a type is to be serialized as an Any.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    try_from = "AnySerde<T>",
    into = "AnySerde<T>",
    bound(
        // TODO: Figure out why this needs clone
        serialize = "T: Serialize + Clone + TypeUrl",
        deserialize = "T: Deserialize<'de> + TypeUrl",
    )
)]
pub struct Any<T>(pub T);

/// Provides a way to convert a type `T` into an [`Any`], even if `T` is itself an [`Any`].
///
/// ```rust
/// # use unionlabs::google::protobuf::duration::Duration;
/// # use unionlabs::google::protobuf::any::{Any, IntoAny};
///
/// let duration = Duration::new(1, 2).expect("valid duration");
/// let _: Any<Duration> = duration.into_any();
/// let _: Any<Duration> = Any(duration).into_any();
/// ```
pub trait IntoAny {
    type T: Encode<Proto> + TypeUrl;
    fn into_any(self) -> Any<Self::T>;
}

impl<T: TypeUrl + Encode<Proto>> IntoAny for T {
    type T = T;

    fn into_any(self) -> Any<Self::T> {
        Any(self)
    }
}

impl<T: TypeUrl + Encode<Proto>> IntoAny for Any<T> {
    type T = T;

    fn into_any(self) -> Any<Self::T> {
        self
    }
}

#[model]
pub struct RawAny {
    pub type_url: String,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub value: Vec<u8>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AnySerde<T> {
    #[serde(rename = "@type")]
    type_url: String,
    #[serde(flatten)]
    data: T,
}

impl<T: TypeUrl> From<Any<T>> for AnySerde<T> {
    fn from(Any(value): Any<T>) -> Self {
        Self {
            type_url: T::type_url(),
            data: value,
        }
    }
}

impl<T: TypeUrl> TryFrom<AnySerde<T>> for Any<T> {
    type Error = String;

    fn try_from(value: AnySerde<T>) -> Result<Self, Self::Error> {
        if value.type_url == T::type_url() {
            Ok(Self(value.data))
        } else {
            Err(format!(
                "incorrect type url, expected `{expected}` but found `{found}`",
                expected = T::type_url(),
                found = value.type_url
            ))
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

impl<T: Encode<Proto> + TypeUrl> From<Any<T>> for RawAny {
    fn from(val: Any<T>) -> Self {
        RawAny {
            type_url: T::type_url().to_string(),
            value: val.0.encode(),
        }
    }
}

impl<T> TryFrom<RawAny> for Any<T>
where
    T: Decode<Proto, Error: core::error::Error> + TypeUrl,
{
    type Error = TryFromAnyError<T>;

    fn try_from(value: RawAny) -> Result<Self, Self::Error> {
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

// NOTE: In order for IntoAny to work, Any cannot implement TypeUrl. If nested Anys are required, you're crazy and I'm not helping you
// impl TypeUrl for protos::google::protobuf::Any {
//     const TYPE_URL: &'static str = "/google.protobuf.Any";
// }

#[derive(DebugNoBound, thiserror::Error)]
pub enum TryFromAnyError<T: Decode<Proto, Error: core::error::Error> + TypeUrl> {
    // TODO: Extract this out into a struct such that it can be reused
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bech32::Bech32, cosmwasm::wasm::msg_instantiate_contract2::MsgInstantiateContract2,
        test_utils::assert_codec_iso,
    };

    #[test]
    fn test_into_any() {
        use crate::google::protobuf::{duration::Duration, timestamp::Timestamp};

        trait Foo {
            type Bar;
        }

        struct A;
        struct B;

        impl Foo for A {
            type Bar = Timestamp;
        }

        impl Foo for B {
            type Bar = Any<Duration>;
        }

        fn wrap_any_one_level<T>(bar: T::Bar) -> Any<<T::Bar as IntoAny>::T>
        where
            T: Foo,
            T::Bar: IntoAny,
        {
            bar.into_any()
        }

        let _: Any<Timestamp> = wrap_any_one_level::<A>(Timestamp {
            seconds: crate::bounded::BoundedI64::new(1).unwrap(),
            nanos: crate::bounded::BoundedI32::new(2).unwrap(),
        });

        let _: Any<Duration> = wrap_any_one_level::<B>(Any(Duration::new(3, 4).unwrap()));
    }

    #[test]
    fn serde() {
        let msg = MsgInstantiateContract2 {
            sender: Bech32::new("union".to_owned(), b"abc".into()),
            admin: Bech32::new("union".to_owned(), b"abc".into()),
            code_id: 1_u64.try_into().unwrap(),
            label: "label".to_owned(),
            msg: b"{}".into(),
            funds: vec![],
            salt: b"salt".into(),
            fix_msg: false,
        };

        let json = serde_json::to_string_pretty(&Any(msg.clone())).unwrap();

        println!("{json}");

        assert_codec_iso::<_, crate::encoding::Json>(&Any(msg));
    }
}
