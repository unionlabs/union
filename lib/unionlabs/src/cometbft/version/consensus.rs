use macros::model;

#[derive(Default)]
#[model(proto(raw(protos::cometbft::version::v1::Consensus), into, from))]
pub struct Consensus {
    #[serde(with = "::serde_utils::string")]
    pub block: u64,
    // REVIEW: Why default?
    #[serde(with = "::serde_utils::string", default)]
    pub app: u64,
}

impl From<protos::cometbft::version::v1::Consensus> for Consensus {
    fn from(value: protos::cometbft::version::v1::Consensus) -> Self {
        Self {
            block: value.block,
            app: value.app,
        }
    }
}

impl From<Consensus> for protos::cometbft::version::v1::Consensus {
    fn from(value: Consensus) -> Self {
        Self {
            block: value.block,
            app: value.app,
        }
    }
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

#[cfg(feature = "ethabi")]
impl From<contracts::glue::TendermintVersionConsensusData> for Consensus {
    fn from(value: contracts::glue::TendermintVersionConsensusData) -> Self {
        Self {
            block: value.block,
            app: value.app,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<Consensus> for contracts::glue::TendermintVersionConsensusData {
    fn from(value: Consensus) -> Self {
        Self {
            block: value.block,
            app: value.app,
        }
    }
}
