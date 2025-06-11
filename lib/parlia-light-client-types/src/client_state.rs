use ibc_union_spec::Duration;
use parlia_types::Valset;
use unionlabs::primitives::{H160, U256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ClientState {
    V1(ClientStateV1),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientStateV1 {
    pub chain_id: U256,
    pub latest_height: u64,

    pub frozen_height: u64,

    /// The unbonding period of the parlia chain this client tracks.
    ///
    /// If a client is not updated for a duration longer than this period, it will expire.
    pub unbond_period: Duration,

    pub ibc_contract_address: H160,

    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub initial_valset: Option<Valset>,
}
