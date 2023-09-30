use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{
        client::height::IsHeight,
        connection::{counterparty::Counterparty, version::Version},
    },
    traits::Id,
    CosmosAccountId, IntoProto, MsgIntoProto, TypeUrl,
};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "
        ClientId: Serialize,
        CounterpartyClientId: Serialize,
        ClientState: Serialize,
    ",
    deserialize = "
        ClientId: for<'d> Deserialize<'d>,
        CounterpartyClientId: for<'d> Deserialize<'d>,
        ClientState: for<'d> Deserialize<'d>,
    ",
))]
pub struct MsgConnectionOpenTry<
    ClientState,
    ClientId,
    CounterpartyClientId,
    ProofHeight: IsHeight,
    ConsensusHeight: IsHeight,
> {
    pub client_id: ClientId,
    pub client_state: ClientState,
    pub counterparty: Counterparty<CounterpartyClientId>,
    pub delay_period: u64,
    pub counterparty_versions: Vec<Version>,
    pub proof_height: ProofHeight,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_init: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_client: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_consensus: Vec<u8>,
    pub consensus_height: ConsensusHeight,
}

impl<
        ClientState: Debug,
        ClientId: Debug,
        CounterpartyClientId: Debug,
        ProofHeight: IsHeight,
        ConsensusHeight: IsHeight,
    > Debug
    for MsgConnectionOpenTry<
        ClientState,
        ClientId,
        CounterpartyClientId,
        ProofHeight,
        ConsensusHeight,
    >
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MsgConnectionOpenTry")
            .field("client_id", &self.client_id)
            .field("client_state", &self.client_state)
            .field("counterparty", &self.counterparty)
            .field("delay_period", &self.delay_period)
            .field("counterparty_versions", &self.counterparty_versions)
            .field("proof_height", &self.proof_height)
            .field("proof_init", &serde_utils::to_hex(&self.proof_init))
            .field("proof_client", &serde_utils::to_hex(&self.proof_client))
            .field(
                "proof_consensus",
                &serde_utils::to_hex(&self.proof_consensus),
            )
            .field("consensus_height", &self.consensus_height)
            .finish()
    }
}

impl TypeUrl for protos::ibc::core::connection::v1::MsgConnectionOpenTry {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.MsgConnectionOpenTry";
}

impl<
        ClientState,
        ClientId,
        CounterpartyClientId,
        ProofHeight: IsHeight,
        ConsensusHeight: IsHeight,
    > MsgIntoProto
    for MsgConnectionOpenTry<
        ClientState,
        ClientId,
        CounterpartyClientId,
        ProofHeight,
        ConsensusHeight,
    >
where
    ClientState: IntoProto<Proto = protos::google::protobuf::Any>,
    ClientId: Id,
    CounterpartyClientId: Id,
{
    type Proto = protos::ibc::core::connection::v1::MsgConnectionOpenTry;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        #[allow(deprecated)]
        Self::Proto {
            client_id: self.client_id.to_string(),
            previous_connection_id: String::new(),
            client_state: Some(self.client_state.into_proto()),
            counterparty: Some(self.counterparty.into()),
            delay_period: self.delay_period,
            counterparty_versions: self
                .counterparty_versions
                .into_iter()
                .map(Into::into)
                .collect(),
            proof_height: Some(self.proof_height.into_height().into()),
            proof_init: self.proof_init,
            proof_client: self.proof_client,
            proof_consensus: self.proof_consensus,
            consensus_height: Some(self.consensus_height.into_height().into()),
            signer: signer.to_string(),
            host_consensus_state_proof: vec![],
        }
    }
}

#[cfg(feature = "ethabi")]
impl<
        ClientState,
        ClientId,
        CounterpartyClientId,
        ProofHeight: IsHeight,
        ConsensusHeight: IsHeight,
    >
    From<
        MsgConnectionOpenTry<
            ClientState,
            ClientId,
            CounterpartyClientId,
            ProofHeight,
            ConsensusHeight,
        >,
    > for contracts::ibc_handler::MsgConnectionOpenTry
where
    ClientState: IntoProto,
    ClientId: Id,
    CounterpartyClientId: Id,
{
    fn from(
        msg: MsgConnectionOpenTry<
            ClientState,
            ClientId,
            CounterpartyClientId,
            ProofHeight,
            ConsensusHeight,
        >,
    ) -> Self {
        Self {
            counterparty: msg.counterparty.into(),
            delay_period: msg.delay_period,
            client_id: msg.client_id.to_string(),
            // TODO(benluelo): Figure out what this is expected to be (i.e. eth abi or proto)
            client_state_bytes: msg.client_state.into_proto_bytes().into(),
            counterparty_versions: msg
                .counterparty_versions
                .into_iter()
                .map(Into::into)
                .collect(),
            proof_init: msg.proof_init.into(),
            proof_client: msg.proof_client.into(),
            proof_consensus: msg.proof_consensus.into(),
            proof_height: msg.proof_height.into_height().into(),
            consensus_height: msg.consensus_height.into_height().into(),
        }
    }
}
