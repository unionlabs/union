use macros::model;

#[model(proto(raw(protos::tendermint::p2p::ProtocolVersion)))]
pub struct ProtocolVersion {
    pub p2p: u64,
    pub block: u64,
    pub app: u64,
}

impl From<protos::tendermint::p2p::ProtocolVersion> for ProtocolVersion {
    fn from(value: protos::tendermint::p2p::ProtocolVersion) -> Self {
        Self {
            p2p: value.p2p,
            block: value.block,
            app: value.app,
        }
    }
}
