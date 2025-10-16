use tendermint_light_client_types::Fraction;
use unionlabs::{
    cosmos::ics23::proof_spec::ProofSpec,
    google::protobuf::duration::Duration,
    ibc::core::client::height::Height,
    primitives::{H160, U256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ClientState {
    V1(ClientStateV1),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientStateV1 {
    // tendermint specific fields
    pub chain_id: String,
    pub trust_level: Fraction,
    pub trusting_period: Duration,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
    pub frozen_height: Option<Height>,
    pub proof_specs: Vec<ProofSpec>,

    /// Execution chain id
    pub evm_chain_id: U256,
    /// Execution height
    pub latest_height: u64,
    /// The IBC contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
}
