use unionlabs::tuple::AsTuple;

pub type ClientState = state_lens_light_client_types::ClientState<Extra>;

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Extra {
    /// the offset at which we extract the u64 timestamp from the l2 consensus state
    /// timestamp = consensus_state[timestamp_offset:timestamp_offset+8]
    pub timestamp_offset: u16,
    /// the offset at which we extract the bytes32 storage root (of the ibc contract on the l2) from the l2 consensus state
    /// storage_root = consensus_state[storage_root_offset:storage_root_offset+32]
    pub storage_root_offset: u16,
}
