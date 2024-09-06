use enumorph::Enumorph;
use queue_msg::queue_msg;
use unionlabs::id::ClientId;

use crate::data::BatchableEvent;

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCall {
    MakeTransactionBatchesWithUpdate(MakeTransactionBatchesWithUpdate),
}

/// Constructs multiple batch transactions, where all of the batches are provable at the new consensus height.
#[queue_msg]
pub struct MakeTransactionBatchesWithUpdate {
    pub client_id: ClientId,
    pub batches: Vec<Vec<BatchableEvent>>,
}
