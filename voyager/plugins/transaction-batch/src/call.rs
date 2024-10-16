use enumorph::Enumorph;
use voyager_message::macros::model;
use unionlabs::id::ClientId;

use crate::data::BatchableEvent;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    MakeTransactionBatchesWithUpdate(MakeTransactionBatchesWithUpdate),
}

/// Constructs multiple batch transactions, where all of the batches are provable at the new consensus height.
#[model]
pub struct MakeTransactionBatchesWithUpdate {
    pub client_id: ClientId,
    pub batches: Vec<Vec<BatchableEvent>>,
}
