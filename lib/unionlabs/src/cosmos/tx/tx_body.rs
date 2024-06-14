use macros::model;

#[model(proto(raw(protos::cosmos::tx::v1beta1::TxBody), into, from))]
pub struct TxBody {
    pub messages: Vec<protos::google::protobuf::Any>,
    pub memo: String,
    pub timeout_height: u64,
    pub extension_options: Vec<protos::google::protobuf::Any>,
    pub non_critical_extension_options: Vec<protos::google::protobuf::Any>,
}

impl From<TxBody> for protos::cosmos::tx::v1beta1::TxBody {
    fn from(value: TxBody) -> Self {
        Self {
            messages: value.messages,
            memo: value.memo,
            timeout_height: value.timeout_height,
            extension_options: value.extension_options,
            non_critical_extension_options: value.non_critical_extension_options,
        }
    }
}

impl From<protos::cosmos::tx::v1beta1::TxBody> for TxBody {
    fn from(value: protos::cosmos::tx::v1beta1::TxBody) -> Self {
        Self {
            messages: value.messages,
            memo: value.memo,
            timeout_height: value.timeout_height,
            extension_options: value.extension_options,
            non_critical_extension_options: value.non_critical_extension_options,
        }
    }
}
