use enumorph::Enumorph;
use ibc_solidity::ibc;
use macros::model;
use subset_of::SubsetOf;
use unionlabs::hash::H256;

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleCall {
    FetchGetLogs(FetchGetLogs),
    MakeFullEvent(MakeFullEvent),
}

/// Fetch all events in `block_number` emitted by the `IBCHandler` via [`eth_getLogs`].
///
/// [`eth_getLogs`]: https://ethereum.org/en/developers/docs/apis/json-rpc/#[model]th_getlogs
#[model]
pub struct FetchGetLogs {
    pub block_number: u64,
    /// If set, only fetch blocks up to this range; otherwise indefinitely unfold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub up_to: Option<u64>,
}

/// Construct a full ChainEvent from the given EVM event and associated metadata.
#[model]
pub struct MakeFullEvent {
    /// The *execution* block number that this event was emitted at.
    pub block_number: u64,
    /// Tx hash of the transaction that emitted this event.
    pub tx_hash: H256,
    pub event: IbcEvents,
}

#[model]
pub enum IbcEvents {
    ClientRegistered(ibc::Ibc::ClientRegistered),
    ClientCreated(ibc::Ibc::ClientCreated),
    ClientUpdated(ibc::Ibc::ClientUpdated),
    ConnectionOpenInit(ibc::Ibc::ConnectionOpenInit),
    ConnectionOpenTry(ibc::Ibc::ConnectionOpenTry),
    ConnectionOpenAck(ibc::Ibc::ConnectionOpenAck),
    ConnectionOpenConfirm(ibc::Ibc::ConnectionOpenConfirm),
    ChannelOpenInit(ibc::Ibc::ChannelOpenInit),
    ChannelOpenTry(ibc::Ibc::ChannelOpenTry),
    ChannelOpenAck(ibc::Ibc::ChannelOpenAck),
    ChannelOpenConfirm(ibc::Ibc::ChannelOpenConfirm),
    ChannelCloseInit(ibc::Ibc::ChannelCloseInit),
    ChannelCloseConfirm(ibc::Ibc::ChannelCloseConfirm),
    SendPacket(ibc::Ibc::SendPacket),
    RecvPacket(ibc::Ibc::RecvPacket),
    RecvIntentPacket(ibc::Ibc::RecvIntentPacket),
    WriteAcknowledgement(ibc::Ibc::WriteAcknowledgement),
    AcknowledgePacket(ibc::Ibc::AcknowledgePacket),
    TimeoutPacket(ibc::Ibc::TimeoutPacket),
}
