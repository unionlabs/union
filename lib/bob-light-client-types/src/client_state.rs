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

    pub l2_finalization_period_seconds: u64,
    pub l2_oracle_address: H160,
    pub l2_oracle_l2_outputs_slot: U256,

    pub frozen_height: u64,

    pub ibc_contract_address: H160,
}
