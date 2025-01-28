use unionlabs::{ibc::core::client::height::Height, primitives::Bytes};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub l1_height: Height,
    pub l2_height: Height,
    /// Proof of the L2 consensus state as stored in the state of the L1.
    pub l2_consensus_state_proof: Bytes,
    pub l2_consensus_state: Bytes,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::{ibc::core::client::height::Height, impl_ethabi_via_try_from_into};

    use crate::Header;

    impl_ethabi_via_try_from_into!(Header => SolHeader);

    alloy::sol! {
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
                l1Height: value.l1_height.height(),
                l2Height: value.l2_height.height(),
                l2InclusionProof: value.l2_consensus_state_proof.into(),
                l2ConsensusState: value.l2_consensus_state.into(),
            }
        }
    }

    impl From<SolHeader> for Header {
        fn from(value: SolHeader) -> Self {
            Self {
                l1_height: Height::new(value.l1Height),
                l2_height: Height::new(value.l2Height),
                l2_consensus_state_proof: value.l2InclusionProof.into(),
                l2_consensus_state: value.l2ConsensusState.into(),
            }
        }
    }
}
