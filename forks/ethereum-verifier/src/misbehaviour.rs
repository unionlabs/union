use crate::{errors::Error, updates::ConsensusUpdate};
use ethereum_consensus::{compute::compute_sync_committee_period_at_slot, context::ChainContext};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Misbehaviour<const SYNC_COMMITTEE_SIZE: usize, CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>> {
    FinalizedHeader(FinalizedHeaderMisbehaviour<SYNC_COMMITTEE_SIZE, CU>),
    NextSyncCommittee(NextSyncCommitteeMisbehaviour<SYNC_COMMITTEE_SIZE, CU>),
}

impl<const SYNC_COMMITTEE_SIZE: usize, CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>>
    Misbehaviour<SYNC_COMMITTEE_SIZE, CU>
{
    pub fn validate_basic<CC: ChainContext>(&self, ctx: &CC) -> Result<(), Error> {
        match self {
            Misbehaviour::FinalizedHeader(data) => data.validate_basic(),
            Misbehaviour::NextSyncCommittee(data) => data.validate_basic(ctx),
        }
    }

    pub fn updates(self) -> (CU, CU) {
        match self {
            Misbehaviour::FinalizedHeader(data) => {
                (data.consensus_update_1, data.consensus_update_2)
            }
            Misbehaviour::NextSyncCommittee(data) => {
                (data.consensus_update_1, data.consensus_update_2)
            }
        }
    }
}

/// FinalizedHeaderMisbehaviour is a misbehaviour that satisfies the followings:
/// 1. Two updates are valid with the consensus state of the client
/// 2. Each finalized header in the two updates has a same finalized slot
/// 3. The two finalized headers are different from each other
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FinalizedHeaderMisbehaviour<
    const SYNC_COMMITTEE_SIZE: usize,
    CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
> {
    pub consensus_update_1: CU,
    pub consensus_update_2: CU,
}

impl<const SYNC_COMMITTEE_SIZE: usize, CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>>
    FinalizedHeaderMisbehaviour<SYNC_COMMITTEE_SIZE, CU>
{
    pub fn validate_basic(&self) -> Result<(), Error> {
        let header_1 = self.consensus_update_1.finalized_beacon_header();
        let header_2 = self.consensus_update_2.finalized_beacon_header();
        if header_1.slot != header_2.slot {
            Err(Error::DifferentSlotInFinalizedHeaderMisbehaviour(
                header_1.slot,
                header_2.slot,
            ))
        } else if header_1 == header_2 {
            Err(Error::SameFinalizedHeaderInFinalizedHeaderMisbehaviour(
                header_1.clone(),
            ))
        } else {
            Ok(())
        }
    }
}

/// NextSyncCommitteeMisbehaviour is a misbehaviour that satisfies the followings:
/// 1. Two updates are valid with the consensus state of the client
/// 2. Each attested header in the two updates has a same period
/// 3. The two next sync committees are different from each other
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NextSyncCommitteeMisbehaviour<
    const SYNC_COMMITTEE_SIZE: usize,
    CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>,
> {
    pub consensus_update_1: CU,
    pub consensus_update_2: CU,
}

impl<const SYNC_COMMITTEE_SIZE: usize, CU: ConsensusUpdate<SYNC_COMMITTEE_SIZE>>
    NextSyncCommitteeMisbehaviour<SYNC_COMMITTEE_SIZE, CU>
{
    pub fn validate_basic<CC: ChainContext>(&self, ctx: &CC) -> Result<(), Error> {
        let period_1 = compute_sync_committee_period_at_slot(
            ctx,
            self.consensus_update_1.attested_beacon_header().slot,
        );
        let period_2 = compute_sync_committee_period_at_slot(
            ctx,
            self.consensus_update_2.attested_beacon_header().slot,
        );
        let next_1 = self
            .consensus_update_1
            .next_sync_committee()
            .ok_or(Error::NoNextSyncCommitteeInNextSyncCommitteeMisbehaviour)?;
        let next_2 = self
            .consensus_update_2
            .next_sync_committee()
            .ok_or(Error::NoNextSyncCommitteeInNextSyncCommitteeMisbehaviour)?;

        if period_1 != period_2 {
            Err(Error::DifferentPeriodInNextSyncCommitteeMisbehaviour(
                period_1, period_2,
            ))
        } else if next_1 == next_2 {
            Err(Error::SameNextSyncCommitteeInNextSyncCommitteeMisbehaviour(
                next_1.aggregate_pubkey.clone(),
            ))
        } else {
            Ok(())
        }
    }
}
