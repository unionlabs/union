#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    /// l1 client id used to check the l2 inclusion proof against
    pub l1_client_id: u32,
    /// l2 chain id
    pub l2_chain_id: String,
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
