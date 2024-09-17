use macros::model;

use crate::google::protobuf::any::RawAny;

#[model(proto(raw(protos::cosmos::tx::v1beta1::TxBody), into, from))]
pub struct TxBody {
    pub messages: Vec<RawAny>,
    pub memo: String,
    pub timeout_height: u64,
    pub extension_options: Vec<RawAny>,
    pub non_critical_extension_options: Vec<RawAny>,
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
        }
    }
}

impl From<protos::cosmos::tx::v1beta1::TxBody> for TxBody {
    fn from(value: protos::cosmos::tx::v1beta1::TxBody) -> Self {
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
        }
    }
}
