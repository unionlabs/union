use crate::phase0::{SignedBeaconBlockHeader, DEPOSIT_CONTRACT_TREE_DEPTH};
use crate::primitives::{
    BlsPublicKey, BlsSignature, Bytes32, CommitteeIndex, Epoch, Gwei, Hash32, Root, Slot,
    ValidatorIndex,
};
use ssz_rs::prelude::*;

#[derive(Default, Clone, Debug, SimpleSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Checkpoint {
    #[serde(with = "crate::serde::as_string")]
    pub epoch: Epoch,
    pub root: Root,
}

#[derive(Default, Clone, Debug, SimpleSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttestationData {
    #[serde(with = "crate::serde::as_string")]
    pub slot: Slot,
    #[serde(with = "crate::serde::as_string")]
    pub index: CommitteeIndex,
    pub beacon_block_root: Root,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexedAttestation<const MAX_VALIDATORS_PER_COMMITTEE: usize> {
    #[serde(with = "crate::serde::collection_over_string")]
    pub attesting_indices: List<ValidatorIndex, MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: BlsSignature,
}

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PendingAttestation<const MAX_VALIDATORS_PER_COMMITTEE: usize> {
    pub aggregation_bits: Bitlist<MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    #[serde(with = "crate::serde::as_string")]
    pub inclusion_delay: Slot,
    #[serde(with = "crate::serde::as_string")]
    pub proposer_index: ValidatorIndex,
}

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attestation<const MAX_VALIDATORS_PER_COMMITTEE: usize> {
    pub aggregation_bits: Bitlist<MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: BlsSignature,
}

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eth1Data {
    pub deposit_root: Root,
    #[serde(with = "crate::serde::as_string")]
    pub deposit_count: u64,
    pub block_hash: Hash32,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DepositMessage {
    #[serde(rename = "pubkey")]
    pub public_key: BlsPublicKey,
    pub withdrawal_credentials: Bytes32,
    #[serde(with = "crate::serde::as_string")]
    pub amount: Gwei,
}

#[derive(Default, Debug, Clone, SimpleSerialize, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DepositData {
    #[serde(rename = "pubkey")]
    pub public_key: BlsPublicKey,
    pub withdrawal_credentials: Bytes32,
    #[serde(with = "crate::serde::as_string")]
    pub amount: Gwei,
    pub signature: BlsSignature,
}

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProposerSlashing {
    pub signed_header_1: SignedBeaconBlockHeader,
    pub signed_header_2: SignedBeaconBlockHeader,
}

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttesterSlashing<const MAX_VALIDATORS_PER_COMMITTEE: usize> {
    pub attestation_1: IndexedAttestation<MAX_VALIDATORS_PER_COMMITTEE>,
    pub attestation_2: IndexedAttestation<MAX_VALIDATORS_PER_COMMITTEE>,
}

const fn get_deposit_proof_length() -> usize {
    DEPOSIT_CONTRACT_TREE_DEPTH + 1
}

const DEPOSIT_PROOF_LENGTH: usize = get_deposit_proof_length();

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Deposit {
    pub proof: Vector<Bytes32, DEPOSIT_PROOF_LENGTH>,
    pub data: DepositData,
}

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VoluntaryExit {
    #[serde(with = "crate::serde::as_string")]
    pub epoch: Epoch,
    #[serde(with = "crate::serde::as_string")]
    pub validator_index: ValidatorIndex,
}

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: BlsSignature,
}
