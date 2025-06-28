use serde::{Deserialize, Serialize};

use crate::indexer::event::types::{
    BlockHash, BlockHeight, BlockTimestamp, EventIndex, TransactionEventIndex, TransactionHash,
    TransactionIndex, UniversalChainId,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Header {
    pub universal_chain_id: UniversalChainId,
    pub block_hash: BlockHash,
    pub height: BlockHeight,
    pub event_index: EventIndex,
    pub timestamp: BlockTimestamp,
    pub transaction_hash: TransactionHash,
    pub transaction_index: TransactionIndex,
    /// deprecated
    pub transaction_event_index: Option<TransactionEventIndex>,
}
