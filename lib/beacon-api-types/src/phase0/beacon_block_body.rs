use unionlabs::primitives::{H256, H768};
#[cfg(feature = "ssz")]
use {
    crate::{
        chain_spec::ChainSpec,
        phase0::{AttestationSsz, AttesterSlashingSsz, DepositSsz},
    },
    ssz::{types::List, Ssz},
};

use crate::phase0::{
    Attestation, AttesterSlashing, Deposit, Eth1Data, ProposerSlashing, SignedVoluntaryExit,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct BeaconBlockBody {
    pub randao_reveal: H768,
    pub eth1_data: Eth1Data,
    pub graffiti: H256,
    pub proposer_slashings: Vec<ProposerSlashing>,
    pub attester_slashings: Vec<AttesterSlashing>,
    pub attestations: Vec<Attestation>,
    pub deposits: Vec<Deposit>,
    pub voluntary_exits: Vec<SignedVoluntaryExit>,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct BeaconBlockBodySsz<C: ChainSpec> {
    pub randao_reveal: H768,
    pub eth1_data: Eth1Data,
    pub graffiti: H256,
    pub proposer_slashings: List<ProposerSlashing, C::MAX_PROPOSER_SLASHINGS>,
    pub attester_slashings: List<AttesterSlashingSsz<C>, C::MAX_ATTESTER_SLASHINGS>,
    pub attestations: List<AttestationSsz<C>, C::MAX_ATTESTATIONS>,
    pub deposits: List<DepositSsz<C>, C::MAX_DEPOSITS>,
    pub voluntary_exits: List<SignedVoluntaryExit, C::MAX_VOLUNTARY_EXITS>,
}
