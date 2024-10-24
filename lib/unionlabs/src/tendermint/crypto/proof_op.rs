use macros::model;

#[model(proto(raw(protos::tendermint::crypto::ProofOp), into, from))]
pub struct ProofOp {
    pub ty: String,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub key: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub data: Vec<u8>,
}

impl From<ProofOp> for protos::tendermint::crypto::ProofOp {
    fn from(value: ProofOp) -> Self {
        Self {
            r#type: value.ty,
            key: value.key,
            data: value.data,
        }
    }
}

impl From<protos::tendermint::crypto::ProofOp> for ProofOp {
    fn from(value: protos::tendermint::crypto::ProofOp) -> Self {
        Self {
            ty: value.r#type,
            key: value.key,
            data: value.data,
        }
    }
}
