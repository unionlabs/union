use enumorph::Enumorph;
use macros::model;
use unionlabs::{ibc::core::client::height::Height, id::ClientId};
use voyager_message::core::ChainId;

use crate::data::BatchableEvent;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    MakeTransactionBatchesWithUpdate(MakeTransactionBatchesWithUpdate),

    MakeMsgConnectionOpenTry(MakeMsgConnectionOpenTry),
    MakeMsgConnectionOpenAck(MakeMsgConnectionOpenAck),
    MakeMsgConnectionOpenConfirm(MakeMsgConnectionOpenConfirm),

    MakeMsgChannelOpenTry(MakeMsgChannelOpenTry),
    MakeMsgChannelOpenAck(MakeMsgChannelOpenAck),
    MakeMsgChannelOpenConfirm(MakeMsgChannelOpenConfirm),

    MakeMsgAcknowledgement(MakeMsgAcknowledgement),
    MakeMsgRecvPacket(MakeMsgRecvPacket),
}

/// Constructs multiple batch transactions, where all of the batches are provable at the new consensus height.
#[model]
pub struct MakeTransactionBatchesWithUpdate {
    pub client_id: ClientId,
    pub batches: Vec<Vec<BatchableEvent>>,
}

#[model]
pub struct MakeMsgConnectionOpenTry {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub connection_open_init_event: voyager_message::ibc_v1::ConnectionOpenInit,
}

#[model]
pub struct MakeMsgConnectionOpenAck {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub connection_open_try_event: voyager_message::ibc_v1::ConnectionOpenTry,
}

#[model]
pub struct MakeMsgConnectionOpenConfirm {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub connection_open_ack_event: voyager_message::ibc_v1::ConnectionOpenAck,
}

#[model]
pub struct MakeMsgChannelOpenTry {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub channel_open_init_event: voyager_message::ibc_v1::ChannelOpenInit,
}

#[model]
pub struct MakeMsgChannelOpenAck {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub channel_open_try_event: voyager_message::ibc_v1::ChannelOpenTry,
}

#[model]
pub struct MakeMsgChannelOpenConfirm {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub channel_open_ack_event: voyager_message::ibc_v1::ChannelOpenAck,
}

#[model]
pub struct MakeMsgRecvPacket {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub send_packet_event: voyager_message::ibc_v1::SendPacket,
}

#[model]
pub struct MakeMsgAcknowledgement {
    /// The chain id of the chain that the event was emitted on.
    pub origin_chain_id: ChainId,
    /// The height to generate the state proofs at.
    pub origin_chain_proof_height: Height,
    /// The chain id of the chain that the message will be sent to.
    pub target_chain_id: ChainId,
    /// The original event that was emitted on the origin chain.
    pub write_acknowledgement_event: voyager_message::ibc_v1::WriteAcknowledgement,
}
