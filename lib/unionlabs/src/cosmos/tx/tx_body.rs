use macros::model;

use crate::google::protobuf::{
    any::RawAny,
    timestamp::{Timestamp, TryFromTimestampError},
};

#[model(proto(raw(protos::cosmos::tx::v1beta1::TxBody), into, from))]
pub struct TxBody {
    pub messages: Vec<RawAny>,
    pub memo: String,
    pub timeout_height: u64,
    pub extension_options: Vec<RawAny>,
    pub non_critical_extension_options: Vec<RawAny>,
    pub unordered: bool,
    pub timeout_timestamp: Option<Timestamp>,
}

impl From<TxBody> for protos::cosmos::tx::v1beta1::TxBody {
    fn from(value: TxBody) -> Self {
        Self {
            messages: value.messages.into_iter().map(Into::into).collect(),
            memo: value.memo,
            timeout_height: value.timeout_height,
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
            unordered: value.unordered,
            timeout_timestamp: value.timeout_timestamp.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromTxBodyError {
    #[error("invalid timeout timestamp")]
    TimeoutTimestamp(#[source] TryFromTimestampError),
}

impl TryFrom<protos::cosmos::tx::v1beta1::TxBody> for TxBody {
    type Error = TryFromTxBodyError;

    fn try_from(value: protos::cosmos::tx::v1beta1::TxBody) -> Result<Self, Self::Error> {
        Ok(Self {
            messages: value.messages.into_iter().map(Into::into).collect(),
            memo: value.memo,
            timeout_height: value.timeout_height,
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
            unordered: value.unordered,
            timeout_timestamp: value
                .timeout_timestamp
                .map(TryInto::try_into)
                .transpose()
                .map_err(TryFromTxBodyError::TimeoutTimestamp)?,
        })
    }
}
