use unionlabs::aptos::account::AccountAddress;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    /// l2 chain id
    pub l2_chain_id: String,
    /// l1 client id used to check the l2 inclusion proof against
    pub l1_client_id: u32,
    /// l2 client id
    pub l2_client_id: u32,
    /// l2 latest height
    pub l2_latest_height: u64,
    /// `aptos_move::table`'s handle that stores commitments on l2
    pub table_handle: AccountAddress,
}
