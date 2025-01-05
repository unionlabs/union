use beacon_api_types::{ForkParameters, PresetBaseKind};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H160, H256},
    uint::U256,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    pub chain_id: U256,
    pub chain_spec: PresetBaseKind,
    pub genesis_validators_root: H256,
    pub genesis_time: u64,
    pub fork_parameters: ForkParameters,
    pub latest_height: u64,
    // even though it would be better to have option, ethabicodec don't handle it as zero struct...
    pub frozen_height: Height,
    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
}
