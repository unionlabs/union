use enumorph::Enumorph;
use ibc_union_spec::ClientId;
use macros::model;
use unionlabs::ibc::core::client::height::Height;
use voyager_sdk::primitives::ChainId;

use crate::data::{BatchableEvent, CommittableEvent};

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    MakeTransactionBatchesWithUpdate(MakeTransactionBatchesWithUpdate),
    MakeProofCommitmentMsg(MakeProofCommitmentMsg),
}

/// Constructs multiple batch transactions, where all of the batches are provable at the new
/// consensus height.
#[model]
pub struct MakeTransactionBatchesWithUpdate {
    pub client_id: ClientId,
    pub batches: Vec<Vec<BatchableEvent>>,
}

/// Make a proof commitment message for an event.
#[model]
pub struct MakeProofCommitmentMsg {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The original event that was emitted on the origin chain.
    pub event: CommittableEvent,
}
