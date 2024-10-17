use macros::model;

#[model(proto(raw(protos::cometbft::types::v1::Data), from, into))]
pub struct Data {
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub txs: Vec<Vec<u8>>,
}

impl From<Data> for protos::cometbft::types::v1::Data {
    fn from(value: Data) -> Self {
        Self { txs: value.txs }
    }
}

impl From<protos::cometbft::types::v1::Data> for Data {
    fn from(value: protos::cometbft::types::v1::Data) -> Self {
        Self { txs: value.txs }
    }
}
