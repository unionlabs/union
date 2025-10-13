use enumorph::Enumorph;
use ibc_union_spec::event::PacketSend;
use macros::model;
use voyager_sdk::primitives::ChainId;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt),
    /// Makes the timeout commitment message.
    MakeMsgTimeoutCommitment(MakeMsgTimeoutCommitment),
    /// Waits until the timeout commitment is complete. Fetches the commitment height and
    /// fetches the timeout commitment at that specific height. Then it makes the timeout message.
    WaitForTimeoutCommitmentAndMakeMsgTimeout(WaitForTimeoutCommitmentAndMakeMsgTimeout),
}

#[model]
pub struct WaitForTimeoutOrReceipt {
    pub event: PacketSend,
    pub sender_chain_id: ChainId,
}

#[model]
pub struct MakeMsgTimeoutCommitment {
    pub event: PacketSend,
    pub sender_chain_id: ChainId,
}

#[model]
pub struct WaitForTimeoutCommitmentAndMakeMsgTimeout {
    pub event: PacketSend,
    pub sender_chain_id: ChainId,
}
