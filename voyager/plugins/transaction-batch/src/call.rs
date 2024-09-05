use enumorph::Enumorph;
use queue_msg::queue_msg;

use crate::data::EventBatch;

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCall {
    MakeMessages(MakeTransactionBatchWithUpdate),
}

#[queue_msg]
pub struct MakeTransactionBatchWithUpdate {
    pub batch: EventBatch,
}
