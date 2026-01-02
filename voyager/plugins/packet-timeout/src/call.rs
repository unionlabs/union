use enumorph::Enumorph;
use ibc_union_spec::{ClientId, Timestamp, event::PacketSend};
use macros::model;
use voyager_sdk::primitives::ChainId;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt),
    MakeMsgTimeout(MakeMsgTimeout),
    UpdateClientToHeightTimestamp(UpdateClientToHeightTimestamp),
    MakeMsgTimeoutFromTrustedHeight(MakeMsgTimeoutFromTrustedHeight),
}

#[model]
pub struct WaitForTimeoutOrReceipt {
    pub event: PacketSend,
    pub chain_id: ChainId,
    pub counterparty_chain_id: ChainId,
}

#[model]
pub struct MakeMsgTimeout {
    pub event: PacketSend,
    pub chain_id: ChainId,
    pub counterparty_chain_id: ChainId,
}

#[model]
pub struct UpdateClientToHeightTimestamp {
    pub chain_id: ChainId,
    pub counterparty_chain_id: ChainId,
    pub client_id: ClientId,
    pub timestamp: Timestamp,
}

#[model]
pub struct MakeMsgTimeoutFromTrustedHeight {
    pub event: PacketSend,
    pub chain_id: ChainId,
    pub counterparty_chain_id: ChainId,
}
