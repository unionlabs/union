use ibc_union_spec::ClientId;
use unionlabs::primitives::{H160, U256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ClientState {
    // compatible with v1.7.2 bedrock contracts
    V1(ClientStateV1),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientStateV1 {
    pub chain_id: U256,
    pub latest_height: u64,
    /// Client id of the client tracking the L1 that the chain this client tracks settles on
    pub l1_client_id: ClientId,
    /// Address of the [`DisputeGameFactory`](https://docs.base.org/base-chain/network-information/base-contracts).
    pub dispute_game_factory_address: H160,
    /// Slot of the [`_disputeGameList`](https://vscode.blockscan.com/ethereum/0x4bba758f006ef09402ef31724203f316ab74e4a0) within the DisputeGameFactory.
    pub dispute_game_factory_dispute_game_list_slot: U256,
    /// Index of the [`rootClaim`](https://vscode.blockscan.com/ethereum/0x08472b9da51fffd935d453c1e4f0436c9d7ad702) (see function `rootClaim`) within the game proxy bytecode.
    pub fault_dispute_game_code_root_claim_index: u32,
    pub frozen_height: u64,
    pub ibc_contract_address: H160,
}
