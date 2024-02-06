use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{
        client::height::IsHeight,
        connection::{counterparty::Counterparty, version::Version},
    },
    TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    bound(
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
    ),
    deny_unknown_fields
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct MsgConnectionOpenTry<
    ClientState,
    ClientId,
    CounterpartyClientId,
    ProofHeight: IsHeight,
    ConsensusHeight: IsHeight,
    ProofInit,
    ProofClient,
    ProofConsensus,
> {
    pub client_id: ClientId,
    pub client_state: ClientState,
    pub counterparty: Counterparty<CounterpartyClientId>,
    pub delay_period: u64,
    pub counterparty_versions: Vec<Version>,
    pub proof_height: ProofHeight,
    pub proof_init: ProofInit,
    pub proof_client: ProofClient,
    pub proof_consensus: ProofConsensus,
    pub consensus_height: ConsensusHeight,
}

impl TypeUrl for protos::ibc::core::connection::v1::MsgConnectionOpenTry {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.MsgConnectionOpenTry";
}
