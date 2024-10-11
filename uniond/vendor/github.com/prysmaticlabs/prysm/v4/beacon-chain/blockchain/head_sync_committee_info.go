package blockchain

import (
	"context"
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// Initialize the state cache for sync committees.
var syncCommitteeHeadStateCache = cache.NewSyncCommitteeHeadState()

// HeadSyncCommitteeFetcher is the interface that wraps the head sync committee related functions.
// The head sync committee functions return callers sync committee indices and public keys with respect to current head state.
type HeadSyncCommitteeFetcher interface {
	HeadSyncCommitteeIndices(ctx context.Context, index primitives.ValidatorIndex, slot primitives.Slot) ([]primitives.CommitteeIndex, error)
	HeadSyncCommitteePubKeys(ctx context.Context, slot primitives.Slot, committeeIndex primitives.CommitteeIndex) ([][]byte, error)
}

// HeadDomainFetcher is the interface that wraps the head sync domain related functions.
// The head sync committee domain functions return callers domain data with respect to slot and head state.
type HeadDomainFetcher interface {
	HeadSyncCommitteeDomain(ctx context.Context, slot primitives.Slot) ([]byte, error)
	HeadSyncSelectionProofDomain(ctx context.Context, slot primitives.Slot) ([]byte, error)
	HeadSyncContributionProofDomain(ctx context.Context, slot primitives.Slot) ([]byte, error)
}

// HeadSyncCommitteeDomain returns the head sync committee domain using current head state advanced up to `slot`.
func (s *Service) HeadSyncCommitteeDomain(ctx context.Context, slot primitives.Slot) ([]byte, error) {
	return s.domainWithHeadState(ctx, slot, params.BeaconConfig().DomainSyncCommittee)
}

// HeadSyncSelectionProofDomain returns the head sync committee domain using current head state advanced up to `slot`.
func (s *Service) HeadSyncSelectionProofDomain(ctx context.Context, slot primitives.Slot) ([]byte, error) {
	return s.domainWithHeadState(ctx, slot, params.BeaconConfig().DomainSyncCommitteeSelectionProof)
}

// HeadSyncContributionProofDomain returns the head sync committee domain using current head state advanced up to `slot`.
func (s *Service) HeadSyncContributionProofDomain(ctx context.Context, slot primitives.Slot) ([]byte, error) {
	return s.domainWithHeadState(ctx, slot, params.BeaconConfig().DomainContributionAndProof)
}

// HeadSyncCommitteeIndices returns the sync committee index position using the head state. Input `slot` is taken in consideration
// where validator's duty for `slot - 1` is used for block inclusion in `slot`. That means when a validator is at epoch boundary
// across EPOCHS_PER_SYNC_COMMITTEE_PERIOD then the validator will be considered using next period sync committee.
//
// Spec definition:
// Being assigned to a sync committee for a given slot means that the validator produces and broadcasts signatures for slot - 1 for inclusion in slot.
// This means that when assigned to an epoch sync committee signatures must be produced and broadcast for slots on range
// [compute_start_slot_at_epoch(epoch) - 1, compute_start_slot_at_epoch(epoch) + SLOTS_PER_EPOCH - 1)
// rather than for the range
// [compute_start_slot_at_epoch(epoch), compute_start_slot_at_epoch(epoch) + SLOTS_PER_EPOCH)
func (s *Service) HeadSyncCommitteeIndices(ctx context.Context, index primitives.ValidatorIndex, slot primitives.Slot) ([]primitives.CommitteeIndex, error) {
	nextSlotEpoch := slots.ToEpoch(slot + 1)
	currentEpoch := slots.ToEpoch(slot)

	switch {
	case slots.SyncCommitteePeriod(nextSlotEpoch) == slots.SyncCommitteePeriod(currentEpoch):
		return s.headCurrentSyncCommitteeIndices(ctx, index, slot)
	// At sync committee period boundary, validator should sample the next epoch sync committee.
	case slots.SyncCommitteePeriod(nextSlotEpoch) == slots.SyncCommitteePeriod(currentEpoch)+1:
		return s.headNextSyncCommitteeIndices(ctx, index, slot)
	default:
		// Impossible condition.
		return nil, errors.New("could get calculate sync subcommittee based on the period")
	}
}

// headCurrentSyncCommitteeIndices returns the input validator `index`'s position indices in the current sync committee with respect to `slot`.
// Head state advanced up to `slot` is used for calculation.
func (s *Service) headCurrentSyncCommitteeIndices(ctx context.Context, index primitives.ValidatorIndex, slot primitives.Slot) ([]primitives.CommitteeIndex, error) {
	headState, err := s.getSyncCommitteeHeadState(ctx, slot)
	if err != nil {
		return nil, err
	}
	return helpers.CurrentPeriodSyncSubcommitteeIndices(headState, index)
}

// headNextSyncCommitteeIndices returns the input validator `index`'s position indices in the next sync committee with respect to `slot`.
// Head state advanced up to `slot` is used for calculation.
func (s *Service) headNextSyncCommitteeIndices(ctx context.Context, index primitives.ValidatorIndex, slot primitives.Slot) ([]primitives.CommitteeIndex, error) {
	headState, err := s.getSyncCommitteeHeadState(ctx, slot)
	if err != nil {
		return nil, err
	}
	return helpers.NextPeriodSyncSubcommitteeIndices(headState, index)
}

// HeadSyncCommitteePubKeys returns the head sync committee public keys with respect to `slot` and subcommittee index `committeeIndex`.
// Head state advanced up to `slot` is used for calculation.
func (s *Service) HeadSyncCommitteePubKeys(ctx context.Context, slot primitives.Slot, committeeIndex primitives.CommitteeIndex) ([][]byte, error) {
	headState, err := s.getSyncCommitteeHeadState(ctx, slot)
	if err != nil {
		return nil, err
	}

	nextSlotEpoch := slots.ToEpoch(headState.Slot() + 1)
	currEpoch := slots.ToEpoch(headState.Slot())

	var syncCommittee *ethpb.SyncCommittee
	if currEpoch == nextSlotEpoch || slots.SyncCommitteePeriod(currEpoch) == slots.SyncCommitteePeriod(nextSlotEpoch) {
		syncCommittee, err = headState.CurrentSyncCommittee()
		if err != nil {
			return nil, err
		}
	} else {
		syncCommittee, err = headState.NextSyncCommittee()
		if err != nil {
			return nil, err
		}
	}

	return altair.SyncSubCommitteePubkeys(syncCommittee, committeeIndex)
}

// returns calculated domain using input `domain` and `slot`.
func (s *Service) domainWithHeadState(ctx context.Context, slot primitives.Slot, domain [4]byte) ([]byte, error) {
	headState, err := s.getSyncCommitteeHeadState(ctx, slot)
	if err != nil {
		return nil, err
	}
	return signing.Domain(headState.Fork(), slots.ToEpoch(headState.Slot()), domain, headState.GenesisValidatorsRoot())
}

// returns the head state that is advanced up to `slot`. It utilizes the cache `syncCommitteeHeadState` by retrieving using `slot` as key.
// For the cache miss, it processes head state up to slot and fill the cache with `slot` as key.
func (s *Service) getSyncCommitteeHeadState(ctx context.Context, slot primitives.Slot) (state.BeaconState, error) {
	var headState state.BeaconState
	var err error
	mLock := async.NewMultilock(fmt.Sprintf("%s-%d", "syncHeadState", slot))
	mLock.Lock()
	defer mLock.Unlock()

	// If there's already a head state exists with the request slot, we don't need to process slots.
	cachedState, err := syncCommitteeHeadStateCache.Get(slot)
	switch {
	case err == nil:
		syncHeadStateHit.Inc()
		headState = cachedState
		return headState, nil
	case errors.Is(err, cache.ErrNotFound):
		headState, err = s.HeadState(ctx)
		if err != nil {
			return nil, err
		}
		if headState == nil || headState.IsNil() {
			return nil, errors.New("nil state")
		}
		headRoot, err := s.HeadRoot(ctx)
		if err != nil {
			return nil, err
		}
		headState, err = transition.ProcessSlotsUsingNextSlotCache(ctx, headState, headRoot, slot)
		if err != nil {
			return nil, err
		}
		syncHeadStateMiss.Inc()
		err = syncCommitteeHeadStateCache.Put(slot, headState)
		return headState, err
	default:
		// In the event, we encounter another error
		// we return it.
		return nil, err
	}
}
