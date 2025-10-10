use enumorph::Enumorph;
use ibc_union_spec::event::PacketSend;
use macros::model;
use voyager_sdk::primitives::ChainId;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt),
    MakeMsgTimeout(MakeMsgTimeout),
}

#[model]
pub struct WaitForTimeoutOrReceipt {
    pub event: PacketSend,
    pub chain_id: ChainId,
}

#[model]
pub struct MakeMsgTimeout {
    pub event: PacketSend,
    pub chain_id: ChainId,
}
