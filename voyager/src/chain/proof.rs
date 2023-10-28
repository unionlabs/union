use std::fmt::Debug;

use futures::Future;
use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::core::client::height::Height,
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
    },
    traits::Chain,
};

pub trait IbcStateRead<Counterparty: Chain, P: IbcPath<Self, Counterparty>>: Chain + Sized
where
    StateProof<P::Output>: Debug + Serialize,
{
    fn state_proof(
        &self,
        path: P,
        at: Self::Height,
    ) -> impl Future<Output = StateProof<P::Output>> + '_;
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StateProof<State> {
    pub state: State,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof: Vec<u8>,
    pub proof_height: Height,
}

impl<State: Debug> Debug for StateProof<State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateProof")
            .field("state", &self.state)
            .field("proof", &serde_utils::to_hex(&self.proof))
            .field("proof_height", &self.proof_height)
            .finish()
    }
}

pub trait IbcStateReadPaths<Counterparty: Chain>:
    Chain
    + IbcStateRead<Counterparty, ClientStatePath<<Self as Chain>::ClientId>>
    + IbcStateRead<
        Counterparty,
        ClientConsensusStatePath<<Self as Chain>::ClientId, Counterparty::Height>,
    > + IbcStateRead<Counterparty, ConnectionPath>
    + IbcStateRead<Counterparty, ChannelEndPath>
    + IbcStateRead<Counterparty, CommitmentPath>
    + IbcStateRead<Counterparty, AcknowledgementPath>
{
}

impl<Counterparty: Chain, T: Chain> IbcStateReadPaths<Counterparty> for T where
    T: IbcStateRead<Counterparty, ClientStatePath<Self::ClientId>>
        + IbcStateRead<Counterparty, ClientConsensusStatePath<Self::ClientId, Counterparty::Height>>
        + IbcStateRead<Counterparty, ConnectionPath>
        + IbcStateRead<Counterparty, ChannelEndPath>
        + IbcStateRead<Counterparty, CommitmentPath>
        + IbcStateRead<Counterparty, AcknowledgementPath>
{
}
