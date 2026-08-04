use enumorph::Enumorph;
use ibc_union_spec::{ClientId, Timestamp, event::PacketSend};
use macros::model;
use unionlabs::ibc::core::client::height::Height;
use voyager_sdk::primitives::ChainId;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt),
    MakeMsgTimeout(MakeMsgTimeout),
    UpdateClientToHeightTimestamp(UpdateClientToHeightTimestamp),
    MakeMsgTimeoutFromTrustedHeight(MakeMsgTimeoutFromTrustedHeight),
    CommitProofLensNonMembershipProof(CommitProofLensNonMembershipProof),
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

/// Commit a non-membership proof (proving that `event.packet` has not been received on
/// `counterparty_chain_id`) onto the L1 that the proof lens client on `chain_id` is anchored to,
/// so that the lens client can subsequently verify it by proxy.
///
/// `counterparty_height` is pinned (rather than re-derived as "latest" on retry) - the
/// commitment is keyed by this exact height, so the client must be refreshed *at this height*
/// (not advanced to a new one) before the timeout message can be built from it.
#[model]
pub struct CommitProofLensNonMembershipProof {
    pub event: PacketSend,
    pub chain_id: ChainId,
    pub counterparty_chain_id: ChainId,
    pub counterparty_height: Height,
}
