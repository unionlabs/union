use serde::{Deserialize, Serialize};

use crate::google::protobuf::{any::RawAny, timestamp::Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxBody<M = RawAny> {
    pub messages: Vec<M>,
    pub memo: String,
    pub timeout_height: u64,
    // lol https://stackoverflow.com/questions/74726116/how-to-skip-serde-serialization-with-skip-serializing-if-for-a-boolean-field
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unordered: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_timestamp: Option<Timestamp>,

    // TODO: Make this struct generic over these types too (in the same way as it is for .messages)
    pub extension_options: Vec<RawAny>,
    pub non_critical_extension_options: Vec<RawAny>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use super::TxBody;
    use crate::{
        google::protobuf::{any::RawAny, timestamp::TryFromTimestampError},
        impl_proto_via_try_from_into, ErrorReporter,
    };

    impl_proto_via_try_from_into!(
        {
            for(M) where
            encode(M: Into<RawAny>)
            decode(RawAny: TryInto<M, Error: core::error::Error>)
        }
        TxBody<M> => protos::cosmos::tx::v1beta1::TxBody
    );

    impl<M: Into<RawAny>> From<TxBody<M>> for protos::cosmos::tx::v1beta1::TxBody {
        fn from(value: TxBody<M>) -> Self {
            Self {
                messages: value
                    .messages
                    .into_iter()
                    .map(Into::<RawAny>::into)
                    .map(Into::into)
                    .collect(),
                memo: value.memo,
                timeout_height: value.timeout_height,
                unordered: value.unordered,
                timeout_timestamp: value.timeout_timestamp.map(Into::into),
                extension_options: value
                    .extension_options
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                non_critical_extension_options: value
                    .non_critical_extension_options
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid timestamp")]
        Timestamp(#[from] TryFromTimestampError),
        // use a string to keep Clone + PartialEq
        #[error("invalid message: {0}")]
        Message(String),
    }

    impl<M> TryFrom<protos::cosmos::tx::v1beta1::TxBody> for TxBody<M>
    where
        RawAny: TryInto<M, Error: core::error::Error>,
    {
        type Error = Error;

        fn try_from(value: protos::cosmos::tx::v1beta1::TxBody) -> Result<Self, Self::Error> {
            Ok(Self {
                messages: value
                    .messages
                    .into_iter()
                    .map(RawAny::from)
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()
                    .map_err(|err| Error::Message(ErrorReporter(err).to_string()))?,
                memo: value.memo,
                timeout_height: value.timeout_height,
                unordered: value.unordered,
                timeout_timestamp: value.timeout_timestamp.map(TryInto::try_into).transpose()?,

                extension_options: value
                    .extension_options
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                non_critical_extension_options: value
                    .non_critical_extension_options
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            })
        }
    }
}
