use std::fmt::Debug;

use prost::Message;

use crate::{IntoProto, TryFromProto, TryFromProtoErrorOf, TypeUrl};

/// Wrapper type to indicate that a type is to be serialized as an Any.
#[derive(Debug, Clone)]
pub struct Any<T>(pub T);

impl<T> From<Any<T>> for protos::google::protobuf::Any
where
    T: IntoProto,
    <T as IntoProto>::Proto: TypeUrl,
{
    fn from(val: Any<T>) -> Self {
        protos::google::protobuf::Any {
            type_url: <T as IntoProto>::Proto::TYPE_URL.to_string(),
            value: val.0.into_proto().encode_to_vec(),
        }
    }
}

impl<T> IntoProto for Any<T>
where
    T: IntoProto,
    <T as IntoProto>::Proto: TypeUrl,
{
    type Proto = protos::google::protobuf::Any;
}

impl TypeUrl for protos::google::protobuf::Any {
    const TYPE_URL: &'static str = "/google.protobuf.Any";
}

#[derive(Debug)]
pub enum TryFromAnyError<T>
where
    T: TryFromProto,
    <T as TryFrom<T::Proto>>::Error: Debug,
{
    IncorrectTypeUrl {
        found: String,
        expected: &'static str,
    },
    Prost(prost::DecodeError),
    TryFromProto(<T as TryFrom<T::Proto>>::Error),
}

impl<T> TryFrom<protos::google::protobuf::Any> for Any<T>
where
    T: TryFromProto,
    T::Proto: TypeUrl + Default,
    <T as TryFrom<T::Proto>>::Error: Debug,
{
    type Error = TryFromAnyError<T>;

    fn try_from(value: protos::google::protobuf::Any) -> Result<Self, Self::Error> {
        if value.type_url == T::Proto::TYPE_URL {
            T::Proto::decode(&*value.value)
                .map_err(TryFromAnyError::Prost)?
                .try_into()
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

impl<T> TryFromProto for Any<T>
where
    T: TryFromProto,
    T::Proto: TypeUrl + Default,
    TryFromProtoErrorOf<T>: Debug,
{
    type Proto = protos::google::protobuf::Any;
}
