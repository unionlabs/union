use serde::{Deserialize, Serialize};

use crate::types::{
    abci_params::AbciParams, block_params::BlockParams, evidence_params::EvidenceParams,
    feature_params::FeatureParams, synchrony_params::SynchronyParams,
    validator_params::ValidatorParams, version_params::VersionParams,
};

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
