use crate::{
    ibc::core::{
        client::height::Height,
        connection::{counterparty::Counterparty, version::Version},
    },
    CosmosAccountId, IntoProto, MsgIntoProto,
};

#[derive(Debug)]
pub struct MsgConnectionOpenTry<ClientState> {
    pub client_id: String,
    pub client_state: ClientState,
    pub counterparty: Counterparty,
    pub delay_period: u64,
    pub counterparty_versions: Vec<Version>,
    pub proof_height: Height,
    pub proof_init: Vec<u8>,
    pub proof_client: Vec<u8>,
    pub proof_consensus: Vec<u8>,
    pub consensus_height: Height,
}

impl<ClientState> MsgIntoProto for MsgConnectionOpenTry<ClientState>
where
    ClientState: IntoProto<Proto = protos::google::protobuf::Any>,
    // <ClientState as IntoProto>::Proto: ,
{
    type Proto = protos::ibc::core::connection::v1::MsgConnectionOpenTry;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        #[allow(deprecated)]
        Self::Proto {
            client_id: self.client_id,
            previous_connection_id: String::new(),
            client_state: Some(self.client_state.into_proto()),
            counterparty: Some(self.counterparty.into()),
            delay_period: self.delay_period,
            counterparty_versions: self
                .counterparty_versions
                .into_iter()
                .map(Into::into)
                .collect(),
            proof_height: Some(self.proof_height.into()),
            proof_init: self.proof_init,
            proof_client: self.proof_client,
            proof_consensus: self.proof_consensus,
            consensus_height: Some(self.consensus_height.into()),
            signer: signer.to_string(),
            host_consensus_state_proof: vec![],
        }
    }
}

#[cfg(feature = "ethabi")]
impl<ClientState> From<MsgConnectionOpenTry<ClientState>>
    for contracts::ibc_handler::MsgConnectionOpenTry
{
    fn from(msg: MsgConnectionOpenTry<ClientState>) -> Self {
        Self {
            counterparty: msg.counterparty.into(),
            delay_period: msg.delay_period,
            client_id: msg.client_id,
            // client_state_bytes: msg.client_state.value.into(),
            // TODO(benluelo): Figure out what this is expected to be (i.e. eth abi or proto)
            client_state_bytes: <_>::default(),
            counterparty_versions: msg
                .counterparty_versions
                .into_iter()
                .map(Into::into)
                .collect(),
            proof_init: msg.proof_init.into(),
            proof_client: msg.proof_client.into(),
            proof_consensus: msg.proof_consensus.into(),
            proof_height: msg.proof_height.into(),
            consensus_height: msg.consensus_height.into(),
        }
    }
}
