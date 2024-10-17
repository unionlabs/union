use macros::model;

#[model(proto(raw(protos::cometbft::crypto::v1::ProofOp), into, from))]
pub struct ProofOp {
    pub ty: String,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub key: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub data: Vec<u8>,
}

impl From<ProofOp> for protos::cometbft::crypto::v1::ProofOp {
    fn from(value: ProofOp) -> Self {
        Self {
            r#type: value.ty,
            key: value.key,
            data: value.data,
        }
    }
}

impl From<protos::cometbft::crypto::v1::ProofOp> for ProofOp {
    fn from(value: protos::cometbft::crypto::v1::ProofOp) -> Self {
        Self {
            ty: value.r#type,
            key: value.key,
            data: value.data,
        }
    }
}
