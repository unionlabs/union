use crate::digest::Digest;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    pub timestamp: u64,
    pub content_digest: Digest,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy::sol! {
        struct SolConsensusState {
            uint64 timestamp;
            bytes32 content_digest;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                timestamp: value.timestamp,
                content_digest: value.content_digest.0.into(),
            }
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum TryFromEthAbiError {}

    impl TryFrom<SolConsensusState> for ConsensusState {
        type Error = TryFromEthAbiError;

        fn try_from(value: SolConsensusState) -> Result<Self, Self::Error> {
            Ok(Self {
                timestamp: value.timestamp,
                content_digest: Digest(value.content_digest.into()),
            })
        }
    }
}
