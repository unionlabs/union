use tendermint_light_client_types::Fraction;
use unionlabs::{
    cosmos::ics23::proof_spec::ProofSpec, google::protobuf::duration::Duration, hash::H160,
    ibc::core::client::height::Height, uint::U256,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub consensus_chain_id: String,
    pub execution_chain_id: U256,

    // TENDERMINT
    pub trust_level: Fraction,
    pub trusting_period: Duration,
    pub max_clock_drift: Duration,
    pub frozen_height: Option<Height>,
    pub latest_height: Height,
    pub proof_specs: Vec<ProofSpec>,
    pub upgrade_path: Vec<String>,

    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
}
