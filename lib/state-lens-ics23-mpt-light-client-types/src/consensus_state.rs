use consensus_primitives::Timestamp;
use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ConsensusState {
    /// Timestamp of the execution layer.
    pub timestamp: Timestamp,
    /// State root of the execution layer.
    pub state_root: H256,
    /// Storage root of the ibc contract extracted from the state root.
    pub storage_root: H256,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use unionlabs::impl_ethabi_via_try_from_into;

    use super::*;

    impl_ethabi_via_try_from_into!(ConsensusState => SolConsensusState);

    alloy::sol! {
        struct SolConsensusState {
            uint64 timestamp;
            bytes32 stateRoot;
            bytes32 storageRoot;
        }
    }

    impl From<ConsensusState> for SolConsensusState {
        fn from(value: ConsensusState) -> Self {
            Self {
                timestamp: value.timestamp.as_nanos(),
                stateRoot: value.state_root.get().into(),
                storageRoot: value.storage_root.get().into(),
            }
        }
    }

    impl From<SolConsensusState> for ConsensusState {
        fn from(value: SolConsensusState) -> Self {
            Self {
                timestamp: Timestamp::from_nanos(value.timestamp),
                state_root: H256::new(value.stateRoot.0),
                storage_root: H256::new(value.storageRoot.0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::hex;
    use unionlabs::{encoding::EthAbi, test_utils::assert_codec_iso_bytes};

    use super::*;

    #[test]
    fn ethabi() {
        // voyager rpc -r voy.run rpc consensus-state 11155111 3 3697397
        // note that this consensus state is technically invalid - this is from before we pushed the fix to the client
        let bz = hex!(
            "000000000000000000000000000000000000000000000000321386383136d3f2" // timestamp
            "00000000000000000000000000000000000000000000000000000000003e7d40" // state_root
            "56013ecdec1cff8ae46d82d3a021ebe40b327a0a6b915383af7756c3fc53797f" // storage_root
        );

        let value = ConsensusState {
            timestamp: Timestamp::from_nanos(3608375302355866610),
            state_root: hex!("00000000000000000000000000000000000000000000000000000000003e7d40")
                .into(),
            storage_root: hex!("56013ecdec1cff8ae46d82d3a021ebe40b327a0a6b915383af7756c3fc53797f")
                .into(),
        };

        assert_codec_iso_bytes::<_, EthAbi>(&value, &bz);
    }
}
