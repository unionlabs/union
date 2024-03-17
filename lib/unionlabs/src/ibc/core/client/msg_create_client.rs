use macros::model;

#[model(proto(raw(protos::ibc::core::client::v1::MsgCreateClient)))]
pub struct MsgCreateClient<ClientState, ConsensusState> {
    pub client_state: ClientState,
    pub consensus_state: ConsensusState,
}
