use ethereum_light_client_types::AccountProof;
use parlia_types::ParliaHeader;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    /// The valset that signed `attestation`.
    pub trusted_valset_epoch_number: u64,
    /// The block that is being attested to as finalized by `attestation`.
    pub source: ParliaHeader,
    /// The block that is being attested to as justified by `attestation`.
    pub target: ParliaHeader,
    /// The block that contains the attestation.
    pub attestation: ParliaHeader,
    /// Proof of the IbcHandler account in the state root of `source`.
    pub ibc_account_proof: AccountProof,
}
