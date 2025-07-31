use ethereum_light_client_types::AccountProof;
use parlia_types::ParliaHeader;

/// Update the client via the Fast Finality ([BEP-126](bep-126)) finality mechanism.
///
/// [bep-126]: https://github.com/bnb-chain/BEPs/blob/master/BEPs/BEP126.md
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    /// The valset that signed `attestation`.
    pub trusted_valset_epoch_number: u64,
    /// The chain of headers to use in the verification. This must contain the attestation, target, and source, and it may additionally contain any additional parents of source that will be verified via an ancestry chain check.
    ///
    /// ```txt
    /// [..ancestors, source, target, attestation]
    /// ```
    pub chain: Vec<ParliaHeader>,
    /// Proof of the IbcHandler account in the state root of `chain[0]`.
    pub ibc_account_proof: AccountProof,
}
