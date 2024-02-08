use serde::{Deserialize, Serialize};

use crate::{Proto, TypeUrl};

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consensus {
    #[serde(with = "::serde_utils::string")]
    pub block: u64,
    // REVIEW: Why default?
    #[serde(with = "::serde_utils::string", default)]
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

impl Proto for Consensus {
    type Proto = protos::tendermint::version::Consensus;
}

impl TypeUrl for protos::tendermint::version::Consensus {
    const TYPE_URL: &'static str = "/tendermint.version.Consensus";
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

#[cfg(feature = "ethabi")]
impl crate::EthAbi for Consensus {
    type EthAbi = contracts::glue::TendermintVersionConsensusData;
}
