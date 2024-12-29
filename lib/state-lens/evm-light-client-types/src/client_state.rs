#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    /// l2 chain id
    pub l2_chain_id: String,
    /// l1 client id used to check the l2 inclusion proof against
    pub l1_client_id: u32,
    /// l2 client id
    pub l2_client_id: u32,
    /// l2 latest height
    pub l2_latest_height: u64,
    /// the offset at which we extract the u64 timestamp from the l2 consensus state
    /// timestamp = consensus_state[timestamp_offset:timestamp_offset+8]
    pub timestamp_offset: u16,
    /// the offset at which we extract the bytes32 state root from the l2 consensus state
    /// state_root = consensus_state[state_root_offset:state_root_offset+32]
    pub state_root_offset: u16,
    /// the offset at which we extract the bytes32 storage root (of the ibc contract on the l2) from the l2 consensus state
    /// storage_root = consensus_state[storage_root_offset:storage_root_offset+32]
    pub storage_root_offset: u16,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::encoding::{Encode, EthAbi};

    use crate::ClientState;

    alloy::sol! {
        struct SolClientState {
            string l2ChainId;
            uint32 l1ClientId;
            uint32 l2ClientId;
            uint64 l2LatestHeight;
            uint16 timestampOffset;
            uint16 stateRootOffset;
            uint16 storageRootOffset;
        }
    }

    impl Encode<EthAbi> for ClientState {
        fn encode(self) -> Vec<u8> {
            SolClientState {
                l2ChainId: self.l2_chain_id,
                l1ClientId: self.l1_client_id,
                l2ClientId: self.l2_client_id,
                l2LatestHeight: self.l2_latest_height,
                timestampOffset: self.timestamp_offset,
                stateRootOffset: self.state_root_offset,
                storageRootOffset: self.storage_root_offset,
            }
            .abi_encode_params()
        }
    }
}
