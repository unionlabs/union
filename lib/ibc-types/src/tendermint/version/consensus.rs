use crate::{FromProto, IntoProto, TypeUrl};

#[derive(Clone, PartialEq)]
pub struct Consensus {
    pub block: u64,
    pub app: u64,
}

impl From<protos::tendermint::version::Consensus> for Consensus {
    fn from(value: protos::tendermint::version::Consensus) -> Self {
        Self {
            block: value.block,
            app: value.app,
        }
    }
}

impl From<Consensus> for protos::tendermint::version::Consensus {
    fn from(value: Consensus) -> Self {
        Self {
            block: value.block,
            app: value.app,
        }
    }
}

impl FromProto for Consensus {
    type Proto = protos::tendermint::version::Consensus;
}

impl IntoProto for Consensus {
    type Proto = protos::tendermint::version::Consensus;
}

impl TypeUrl for protos::tendermint::version::Consensus {
    const TYPE_URL: &'static str = "/tendermint.version.Consensus";
}
