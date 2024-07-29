use enumorph::Enumorph;
use queue_msg::queue_msg;
use unionlabs::ibc::core::client::height::Height;

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleFetch {
    FetchEvents(FetchEvents),
    FetchGetLogs(FetchGetLogs),
    FetchBeaconBlockRange(FetchBeaconBlockRange),
}

#[queue_msg]
pub struct FetchEvents {
    pub from_height: Height,
    pub to_height: Height,
}

#[queue_msg]
pub struct FetchGetLogs {
    pub from_slot: u64,
    pub to_slot: u64,
}

/// NOTE: This isn't just fetching one block because sometimes beacon slots are missed. We need to be able to fetch a range of slots to account for this.
/// The range is `[from_slot..to_slot)`, so to fetch a single block `N`, the range would be `N..N+1`.
#[queue_msg]
pub struct FetchBeaconBlockRange {
    pub from_slot: u64,
    pub to_slot: u64,
}
