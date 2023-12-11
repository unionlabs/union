use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{
        client::height::IsHeight,
        connection::{counterparty::Counterparty, version::Version},
    },
    TypeUrl,
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
