use crate::state::{SyncCommitteeKeeper, SyncCommitteeView};
use ethereum_consensus::sync_protocol::SyncCommittee;
use ethereum_consensus::{
    beacon::{BeaconBlockHeader, Slot},
    types::H256,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MockStore<const SYNC_COMMITTEE_SIZE: usize> {
    pub latest_finalized_header: BeaconBlockHeader,
    pub latest_execution_root: H256,
    pub current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    pub next_sync_committee: Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>,
}

impl<const SYNC_COMMITTEE_SIZE: usize> MockStore<SYNC_COMMITTEE_SIZE> {
    pub fn new(
        header: BeaconBlockHeader,
        current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
        execution_state_root: H256,
    ) -> Self {
        Self {
            latest_finalized_header: header,
            latest_execution_root: execution_state_root,
            current_sync_committee,
            next_sync_committee: None,
        }
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> SyncCommitteeView<SYNC_COMMITTEE_SIZE>
    for MockStore<SYNC_COMMITTEE_SIZE>
{
    fn current_slot(&self) -> Slot {
        self.latest_finalized_header.slot
    }

    fn current_sync_committee(&self) -> &SyncCommittee<SYNC_COMMITTEE_SIZE> {
        &self.current_sync_committee
    }

    fn next_sync_committee(&self) -> Option<&SyncCommittee<SYNC_COMMITTEE_SIZE>> {
        self.next_sync_committee.as_ref()
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> SyncCommitteeKeeper<SYNC_COMMITTEE_SIZE>
    for MockStore<SYNC_COMMITTEE_SIZE>
{
    fn set_finalized_header(&mut self, header: BeaconBlockHeader) {
        self.latest_finalized_header = header;
    }
    fn set_current_sync_committee(
        &mut self,
        current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    ) {
        self.current_sync_committee = current_sync_committee;
    }
    fn set_next_sync_committee(
        &mut self,
        next_sync_committee: Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>,
    ) {
        self.next_sync_committee = next_sync_committee;
    }
}
