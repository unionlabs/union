use std::{fmt::Debug, marker::PhantomData};

use prost::Message;
use serde::{
    de::{self, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

use crate::{
    CosmosAccountId, IntoProto, MsgIntoProto, Proto, TryFromProto, TryFromProtoBytesError,
    TryFromProtoErrorOf, TypeUrl,
};

/// Wrapper type to indicate that a type is to be serialized as an Any.
#[derive(Debug, Clone, PartialEq)]
pub struct Any<T>(pub T);

/// TODO(unionlabs/union#876): Properly implement google.protobuf.Any json serde
impl<'de, T> Deserialize<'de> for Any<T>
where
    T: Deserialize<'de> + TryFromProto,
    <T as Proto>::Proto: TypeUrl,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct AnyVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for AnyVisitor<T>
        where
            T: Deserialize<'de> + TryFromProto,
            <T as Proto>::Proto: TypeUrl,
        {
            type Value = Any<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "a google.protobuf.Any containing {}",
                    T::Proto::TYPE_URL
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
                            return Err(de::Error::unknown_field(unknown, &["type_url", "data"]))
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

        deserializer.deserialize_struct("Any", &["type_url", "data"], AnyVisitor::<T>(PhantomData))
    }
}

impl<T> Serialize for Any<T>
where
    T: Serialize + IntoProto,
    <T as Proto>::Proto: TypeUrl,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Any", 2)?;
        s.serialize_field("type_url", T::Proto::TYPE_URL)?;
        s.serialize_field("data", &self.0)?;
        s.end()
    }
}

impl<T> From<Any<T>> for protos::google::protobuf::Any
where
    T: IntoProto,
    <T as Proto>::Proto: TypeUrl,
{
    fn from(val: Any<T>) -> Self {
        protos::google::protobuf::Any {
            type_url: <T as Proto>::Proto::TYPE_URL.to_string(),
            value: val.0.into_proto().encode_to_vec(),
        }
    }
}

impl<T> Proto for Any<T>
where
    T: IntoProto,
    <T as Proto>::Proto: TypeUrl,
{
    type Proto = protos::google::protobuf::Any;
}

impl<T> MsgIntoProto for Any<T>
where
    T: MsgIntoProto,
    <T as MsgIntoProto>::Proto: TypeUrl,
{
    type Proto = protos::google::protobuf::Any;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        protos::google::protobuf::Any {
            type_url: <T as MsgIntoProto>::Proto::TYPE_URL.to_string(),
            value: self.0.into_proto_with_signer(signer).encode_to_vec(),
        }
    }
}

impl TypeUrl for protos::google::protobuf::Any {
    const TYPE_URL: &'static str = "/google.protobuf.Any";
}

#[derive(Debug)]
pub enum TryFromAnyError<T>
where
    T: TryFromProto,
    T::Proto: TypeUrl,
    <T as TryFrom<T::Proto>>::Error: Debug,
{
    IncorrectTypeUrl {
        found: String,
        expected: &'static str,
    },
    TryFromProto(TryFromProtoBytesError<TryFromProtoErrorOf<T>>),
}

impl<T> TryFrom<protos::google::protobuf::Any> for Any<T>
where
    T: TryFromProto,
    T::Proto: TypeUrl,
    // REVIEW: Is this bound required?
    TryFromProtoErrorOf<T>: Debug,
{
    type Error = TryFromAnyError<T>;

    fn try_from(value: protos::google::protobuf::Any) -> Result<Self, Self::Error> {
        if value.type_url == T::Proto::TYPE_URL {
            T::try_from_proto_bytes(&value.value)
                .map_err(TryFromAnyError::TryFromProto)
                .map(Any)
        } else {
            Err(TryFromAnyError::IncorrectTypeUrl {
                found: value.type_url,
                expected: T::Proto::TYPE_URL,
            })
        }
    }
}
