use serde::{Deserialize, Serialize};

use crate::types::{
    block_params::BlockParams, evidence_params::EvidenceParams, synchrony_params::SynchronyParams,
};
use crate::types::validator_params::ValidatorParams;
use crate::types::version_params::VersionParams;
use crate::types::abci_params::AbciParams;
use crate::types::feature_params::FeatureParams;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsensusParams {
    pub block: BlockParams,
    pub evidence: EvidenceParams,
    pub validator: ValidatorParams,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<VersionParams>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abci: Option<AbciParams>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synchrony: Option<SynchronyParams>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature: Option<FeatureParams>,
}

// TODO: Implement proto
// #[cfg(feature = "proto")]
// pub mod proto {}
