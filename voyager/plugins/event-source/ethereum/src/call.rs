use chain_utils::ethereum::IBCHandlerEvents;
use enumorph::Enumorph;
use subset_of::SubsetOf;
use unionlabs::hash::H256;
use voyager_message::macros::model;

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleCall {
    FetchBlock(FetchBlock),
    FetchGetLogs(FetchGetLogs),
    MakeFullEvent(MakeFullEvent),
}

/// Fetch events in a beacon block. This is a separate step from [`FetchGetLogs`] since beacon slots may be missed.
#[model]
pub struct FetchBlock {
    pub slot: u64,
}

/// Fetch all events in `block_number` emitted by the `IBCHandler` via [`eth_getLogs`].
///
/// [`eth_getLogs`]: https://ethereum.org/en/developers/docs/apis/json-rpc/#[model]th_getlogs
#[model]
pub struct FetchGetLogs {
    pub block_number: u64,
}

/// Construct a full ChainEvent from the given EVM event and associated metadata.
#[model]
pub struct MakeFullEvent {
    /// The *execution* block number that this event was emitted at.
    pub block_number: u64,
    /// Tx hash of the transaction that emitted this event.
    pub tx_hash: H256,
    pub event: ibc_solidity::ibc::Ibc::IbcEvents,
}
