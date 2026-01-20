use unionlabs_primitives::Bytes;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub l1_height: u64,
    pub l2_height: u64,
    /// Proof of the L2 consensus state as stored in the state of the L1.
    pub l2_consensus_state_proof: Bytes,
    pub l2_consensus_state: Bytes,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use unionlabs_encoding::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(Header => SolHeader);

    alloy_sol_types::sol! {
        struct SolHeader {
            uint64 l1Height;
            uint64 l2Height;
            bytes l2InclusionProof;
            bytes l2ConsensusState;
        }
    }

    impl From<Header> for SolHeader {
        fn from(value: Header) -> Self {
            Self {
                l1Height: value.l1_height,
                l2Height: value.l2_height,
                l2InclusionProof: value.l2_consensus_state_proof.into(),
                l2ConsensusState: value.l2_consensus_state.into(),
            }
        }
    }

    impl From<SolHeader> for Header {
        fn from(value: SolHeader) -> Self {
            Self {
                l1_height: value.l1Height,
                l2_height: value.l2Height,
                l2_consensus_state_proof: value.l2InclusionProof.into(),
                l2_consensus_state: value.l2ConsensusState.into(),
            }
        }
    }
}
