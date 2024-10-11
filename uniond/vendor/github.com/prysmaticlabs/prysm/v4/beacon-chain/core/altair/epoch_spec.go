package altair

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	log "github.com/sirupsen/logrus"
)

// ProcessSyncCommitteeUpdates  processes sync client committee updates for the beacon state.
//
// nolint:dupword
// Spec code:
// def process_sync_committee_updates(state: BeaconState) -> None:
//
//	next_epoch = get_current_epoch(state) + Epoch(1)
//	if next_epoch % EPOCHS_PER_SYNC_COMMITTEE_PERIOD == 0:
//	    state.current_sync_committee = state.next_sync_committee
//	    state.next_sync_committee = get_next_sync_committee(state)
func ProcessSyncCommitteeUpdates(ctx context.Context, beaconState state.BeaconState) (state.BeaconState, error) {
	nextEpoch := time.NextEpoch(beaconState)
	if nextEpoch%params.BeaconConfig().EpochsPerSyncCommitteePeriod == 0 {
		nextSyncCommittee, err := beaconState.NextSyncCommittee()
		if err != nil {
			return nil, err
		}
		if err := beaconState.SetCurrentSyncCommittee(nextSyncCommittee); err != nil {
			return nil, err
		}
		nextSyncCommittee, err = NextSyncCommittee(ctx, beaconState)
		if err != nil {
			return nil, err
		}
		if err := beaconState.SetNextSyncCommittee(nextSyncCommittee); err != nil {
			return nil, err
		}
		if err := helpers.UpdateSyncCommitteeCache(beaconState); err != nil {
			log.WithError(err).Error("Could not update sync committee cache")
		}
	}
	return beaconState, nil
}

// ProcessParticipationFlagUpdates processes participation flag updates by rotating current to previous.
//
// nolint:dupword
// Spec code:
// def process_participation_flag_updates(state: BeaconState) -> None:
//
//	state.previous_epoch_participation = state.current_epoch_participation
//	state.current_epoch_participation = [ParticipationFlags(0b0000_0000) for _ in range(len(state.validators))]
func ProcessParticipationFlagUpdates(beaconState state.BeaconState) (state.BeaconState, error) {
	c, err := beaconState.CurrentEpochParticipation()
	if err != nil {
		return nil, err
	}
	if err := beaconState.SetPreviousParticipationBits(c); err != nil {
		return nil, err
	}
	if err := beaconState.SetCurrentParticipationBits(make([]byte, beaconState.NumValidators())); err != nil {
		return nil, err
	}
	return beaconState, nil
}
