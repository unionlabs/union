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
    use unionlabs::encoding::{Encode, EthAbi};

    use crate::Header;

    impl Encode<EthAbi> for Header {
        fn encode(self) -> Vec<u8> {
            Into::<SolHeader>::into(self).abi_encode_params()
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {}

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
}
