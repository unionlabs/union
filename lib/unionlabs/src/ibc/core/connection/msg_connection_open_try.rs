use macros::model;
use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{
        client::height::IsHeight,
        connection::{counterparty::Counterparty, version::Version},
    },
    traits::Id,
};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenTry)))]
#[serde(bound(
    serialize = "
        ClientId: Serialize,
        CounterpartyClientId: Serialize,
        ClientState: Serialize,
        ProofInit: Serialize,
        ProofClient: Serialize,
        ProofConsensus: Serialize,
    ",
    deserialize = "
        ClientId: for<'d> Deserialize<'d>,
        CounterpartyClientId: for<'d> Deserialize<'d>,
        ClientState: for<'d> Deserialize<'d>,
        ProofInit: for<'d> Deserialize<'d>,
        ProofClient: for<'d> Deserialize<'d>,
        ProofConsensus: for<'d> Deserialize<'d>,
    ",
))]
pub struct MsgConnectionOpenTry<
    ClientState,
    ClientId: Id,
    CounterpartyClientId: Id,
    CounterpartyConnectionId: Id,
    ProofHeight: IsHeight,
    ConsensusHeight: IsHeight,
    ProofInit,
    ProofClient,
    ProofConsensus,
> {
    pub client_id: ClientId,
    pub client_state: ClientState,
    pub counterparty: Counterparty<CounterpartyClientId, CounterpartyConnectionId>,
    pub delay_period: u64,
    pub counterparty_versions: Vec<Version>,
    pub proof_height: ProofHeight,
    pub proof_init: ProofInit,
    pub proof_client: ProofClient,
    pub proof_consensus: ProofConsensus,
    pub consensus_height: ConsensusHeight,
}
