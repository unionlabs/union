use enumorph::Enumorph;
use ibc_union_spec::ClientId;
use macros::model;
#[cfg(doc)]
use voyager_sdk::{
    message::data::{IbcDatagram, OrderedHeaders},
    vm::Op,
};

use crate::data::{BatchableEvent, UnsaturatedMsgWithStoreKey};

#[model]
#[derive(Enumorph)]
pub enum ModuleCallback {
    MakeIbcMessagesFromUpdate(MakeIbcMessagesFromUpdate),
    MakeBatchTransaction(MakeBatchTransaction),
    MakeProofLensClientUpdateWithMessages(MakeProofLensClientUpdateWithMessages),
}

/// Given an [`OrderedHeaders`], returns [`Op`]s that generate [`IbcDatagram`]s with proofs at the
/// highest height of the updates.
#[model]
pub struct MakeIbcMessagesFromUpdate {
    pub client_id: ClientId,
    pub batches: Vec<Vec<BatchableEvent>>,
}

#[model]
pub struct MakeBatchTransaction {
    // NOTE: We could technically fetch this from the information in the callback data messages,
    // but this is just so much easier
    pub client_id: ClientId,
}

#[model]
pub struct MakeProofLensClientUpdateWithMessages {
    pub client_id: ClientId,
    pub unsaturated_msgs: Vec<UnsaturatedMsgWithStoreKey>,
}
