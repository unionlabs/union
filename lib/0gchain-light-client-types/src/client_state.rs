use tendermint_light_client_types::Fraction;
use unionlabs::{google::protobuf::duration::Duration, primitives::H160};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    /// consensus client
    pub l1_client_id: u32,
    /// execution chain id
    pub chain_id: String,
    /// execution height
    pub latest_height: u64,
    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
    pub trust_level: Fraction,
    pub trusting_period: Duration,
    pub max_clock_drift: Duration,
}
