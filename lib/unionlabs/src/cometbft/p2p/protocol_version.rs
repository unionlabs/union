use macros::model;

#[model(proto(raw(protos::cometbft::p2p::v1::ProtocolVersion)))]
pub struct ProtocolVersion {
    pub p2p: u64,
    pub block: u64,
    pub app: u64,
}

impl From<protos::cometbft::p2p::v1::ProtocolVersion> for ProtocolVersion {
    fn from(value: protos::cometbft::p2p::v1::ProtocolVersion) -> Self {
        Self {
            p2p: value.p2p,
            block: value.block,
            app: value.app,
        }
    }
}
