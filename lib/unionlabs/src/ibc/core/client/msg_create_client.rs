use macros::proto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::ibc::core::client::v1::MsgCreateClient)]
pub struct MsgCreateClient<ClientState, ConsensusState> {
    pub client_state: ClientState,
    pub consensus_state: ConsensusState,
}
