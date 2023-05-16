use crate::{errors::Error, updates::ConsensusUpdate};
use ethereum_consensus::{
    beacon::{BeaconBlockHeader, Slot},
    compute::compute_sync_committee_period_at_slot,
    context::ChainContext,
    sync_protocol::SyncCommittee,
};

/// NextSyncCommitteeView is a view corresponds to a specific light client update
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NextSyncCommitteeView<const SYNC_COMMITTEE_SIZE: usize> {
    /// finalized header's slot from light client update
    pub current_slot: Slot,
    /// next sync committee from light client update
    pub next_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
}

pub trait SyncCommitteeView<const SYNC_COMMITTEE_SIZE: usize> {
    fn current_slot(&self) -> Slot;
    fn current_sync_committee(&self) -> &SyncCommittee<SYNC_COMMITTEE_SIZE>;
    fn next_sync_committee(&self) -> Option<&SyncCommittee<SYNC_COMMITTEE_SIZE>>;
}

pub trait SyncCommitteeKeeper<const SYNC_COMMITTEE_SIZE: usize> {
    fn set_finalized_header(&mut self, header: BeaconBlockHeader);
    fn set_current_sync_committee(
        &mut self,
        current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    );
    fn set_next_sync_committee(
        &mut self,
        next_sync_committee: Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>,
    );
}

pub fn apply_sync_committee_update<
    const SYNC_COMMITTEE_SIZE: usize,
    CC: ChainContext,
    S: SyncCommitteeView<SYNC_COMMITTEE_SIZE> + SyncCommitteeKeeper<SYNC_COMMITTEE_SIZE>,
    CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
>(
    ctx: &CC,
    state: &mut S,
    consensus_update: &CU,
) -> Result<bool, Error> {
    let mut updated = false;
    let (update_current, update_next) =
        should_update_sync_committees(ctx, state, consensus_update)?;
    if let Some(update_current) = update_current {
        state.set_current_sync_committee(update_current.clone());
        updated = true;
    }
    if let Some(update_next) = update_next {
        state.set_next_sync_committee(update_next.map(|c| c.clone()));
        updated = true;
    }
    if consensus_update.finalized_beacon_header().slot > state.current_slot() {
        state.set_finalized_header(consensus_update.finalized_beacon_header().clone());
        updated = true;
    }
    Ok(updated)
}

fn should_update_sync_committees<
    's,
    'u,
    const SYNC_COMMITTEE_SIZE: usize,
    CC: ChainContext,
    S: SyncCommitteeView<SYNC_COMMITTEE_SIZE>,
    CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
>(
    ctx: &CC,
    state: &'s S,
    consensus_update: &'u CU,
) -> Result<
    (
        Option<&'s SyncCommittee<SYNC_COMMITTEE_SIZE>>,
        Option<Option<&'u SyncCommittee<SYNC_COMMITTEE_SIZE>>>,
    ),
    Error,
> {
    let store_period = compute_sync_committee_period_at_slot(ctx, state.current_slot());
    let update_finalized_period =
        compute_sync_committee_period_at_slot(ctx, consensus_update.finalized_beacon_header().slot);

    if store_period != update_finalized_period && store_period + 1 != update_finalized_period {
        return Err(Error::InvalidFinalizedPeriod(
            store_period,
            update_finalized_period,
            "finalized period must be equal to store_period or store_period+1".into(),
        ));
    }

    if let Some(store_next_sync_committee) = state.next_sync_committee() {
        if update_finalized_period == store_period + 1 {
            Ok((
                Some(store_next_sync_committee),
                Some(consensus_update.next_sync_committee()),
            ))
        } else {
            // no updates
            Ok((None, None))
        }
    } else if update_finalized_period == store_period {
        Ok((None, Some(consensus_update.next_sync_committee())))
    } else {
        Err(Error::CannotRotateNextSyncCommittee(
            store_period,
            update_finalized_period,
        ))
    }
}
