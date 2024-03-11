use macros::proto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::tendermint::version::Consensus, into, from)]
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
