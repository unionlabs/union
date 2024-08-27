use std::num::NonZeroU32;

use enumorph::Enumorph;
use queue_msg::queue_msg;
use unionlabs::{ibc::core::client::height::Height, id::ConnectionId};
use voyager_message::callback::InfoOrMeta;

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchBlocks(FetchBlocks),
    FetchTransactions(FetchTransactions),
    FetchClientFromConnectionId(FetchClientFromConnectionId),
}

#[queue_msg]
pub struct FetchBlocks {
    pub from_height: Height,
    pub to_height: Height,
}

#[queue_msg]
pub struct FetchTransactions {
    pub height: Height,
    pub page: NonZeroU32,
}

#[queue_msg]
pub struct FetchClientFromConnectionId {
    pub connection_id: ConnectionId,
    pub fetch_type: InfoOrMeta,
}
