use unionlabs::tuple::AsTuple;

pub type ClientState = state_lens_light_client_types::ClientState<Extra>;

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Extra {
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

#[cfg(test)]
mod tests {
    use alloy::dyn_abi::SolType;

    use super::*;

    #[test]
    fn ethabi() {
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

        let bz = alloy::hex::decode("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000072734500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000083131313535313131000000000000000000000000000000000000000000000000").unwrap();

        SolClientState::abi_decode(&bz, true).unwrap();
        SolClientState::abi_decode_params(&bz, true).unwrap_err();
    }
}
