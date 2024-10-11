package stategen

import (
	"context"
	stderrors "errors"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
)

var ErrNoDataForSlot = errors.New("cannot retrieve data for slot")

// HasState returns true if the state exists in cache or in DB.
func (s *State) HasState(ctx context.Context, blockRoot [32]byte) (bool, error) {
	has, err := s.hasStateInCache(ctx, blockRoot)
	if err != nil {
		return false, err
	}
	if has {
		return true, nil
	}
	return s.beaconDB.HasState(ctx, blockRoot), nil
}

// hasStateInCache returns true if the state exists in cache.
func (s *State) hasStateInCache(_ context.Context, blockRoot [32]byte) (bool, error) {
	if s.hotStateCache.has(blockRoot) {
		return true, nil
	}
	_, has, err := s.epochBoundaryStateCache.getByBlockRoot(blockRoot)
	if err != nil {
		return false, err
	}
	return has, nil
}

// StateByRootIfCachedNoCopy retrieves a state using the input block root only if the state is already in the cache.
func (s *State) StateByRootIfCachedNoCopy(blockRoot [32]byte) state.BeaconState {
	if !s.hotStateCache.has(blockRoot) {
		return nil
	}
	return s.hotStateCache.getWithoutCopy(blockRoot)
}

// StateByRoot retrieves the state using input block root.
func (s *State) StateByRoot(ctx context.Context, blockRoot [32]byte) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "stateGen.StateByRoot")
	defer span.End()

	// Genesis case. If block root is zero hash, short circuit to use genesis state stored in DB.
	if blockRoot == params.BeaconConfig().ZeroHash {
		root, err := s.beaconDB.GenesisBlockRoot(ctx)
		if err != nil {
			return nil, stderrors.Join(ErrNoGenesisBlock, err)
		}
		blockRoot = root
	}
	return s.loadStateByRoot(ctx, blockRoot)
}

// ActiveNonSlashedBalancesByRoot retrieves the effective balances of all active and non-slashed validators at the
// state with a given root
func (s *State) ActiveNonSlashedBalancesByRoot(ctx context.Context, blockRoot [32]byte) ([]uint64, error) {
	st, err := s.StateByRoot(ctx, blockRoot)
	if err != nil {
		return nil, err
	}
	if st == nil || st.IsNil() {
		return nil, errNilState
	}
	epoch := time.CurrentEpoch(st)

	balances := make([]uint64, st.NumValidators())
	var balanceAccretor = func(idx int, val state.ReadOnlyValidator) error {
		if helpers.IsActiveNonSlashedValidatorUsingTrie(val, epoch) {
			balances[idx] = val.EffectiveBalance()
		} else {
			balances[idx] = 0
		}
		return nil
	}
	if err := st.ReadFromEveryValidator(balanceAccretor); err != nil {
		return nil, err
	}
	return balances, nil
}

// StateByRootInitialSync retrieves the state from the DB for the initial syncing phase.
// It assumes initial syncing using a block list rather than a block tree hence the returned
// state is not copied (block batches returned from initial sync are linear).
// It invalidates cache for parent root because pre-state will get mutated.
//
// WARNING: Do not use this method for anything other than initial syncing purpose or block tree is applied.
func (s *State) StateByRootInitialSync(ctx context.Context, blockRoot [32]byte) (state.BeaconState, error) {
	// Genesis case. If block root is zero hash, short circuit to use genesis state stored in DB.
	if blockRoot == params.BeaconConfig().ZeroHash {
		return s.beaconDB.GenesisState(ctx)
	}

	// To invalidate cache for parent root because pre-state will get mutated.
	// It is a parent root because StateByRootInitialSync is always used to fetch the block's parent state.
	defer s.hotStateCache.delete(blockRoot)

	if s.hotStateCache.has(blockRoot) {
		return s.hotStateCache.getWithoutCopy(blockRoot), nil
	}

	cachedInfo, ok, err := s.epochBoundaryStateCache.getByBlockRoot(blockRoot)
	if err != nil {
		return nil, err
	}
	if ok {
		return cachedInfo.state, nil
	}

	startState, err := s.latestAncestor(ctx, blockRoot)
	if err != nil {
		return nil, errors.Wrap(err, "could not get ancestor state")
	}
	if startState == nil || startState.IsNil() {
		return nil, errUnknownState
	}
	summary, err := s.stateSummary(ctx, blockRoot)
	if err != nil {
		return nil, errors.Wrap(err, "could not get state summary")
	}
	if startState.Slot() == summary.Slot {
		return startState, nil
	}

	blks, err := s.loadBlocks(ctx, startState.Slot()+1, summary.Slot, bytesutil.ToBytes32(summary.Root))
	if err != nil {
		return nil, errors.Wrap(err, "could not load blocks")
	}
	startState, err = s.replayBlocks(ctx, startState, blks, summary.Slot)
	if err != nil {
		return nil, errors.Wrap(err, "could not replay blocks")
	}

	return startState, nil
}

// This returns the state summary object of a given block root. It first checks the cache, then checks the DB.
func (s *State) stateSummary(ctx context.Context, blockRoot [32]byte) (*ethpb.StateSummary, error) {
	var summary *ethpb.StateSummary
	var err error

	summary, err = s.beaconDB.StateSummary(ctx, blockRoot)
	if err != nil {
		return nil, err
	}

	if summary == nil {
		return s.recoverStateSummary(ctx, blockRoot)
	}
	return summary, nil
}

// RecoverStateSummary recovers state summary object of a given block root by using the saved block in DB.
func (s *State) recoverStateSummary(ctx context.Context, blockRoot [32]byte) (*ethpb.StateSummary, error) {
	if s.beaconDB.HasBlock(ctx, blockRoot) {
		b, err := s.beaconDB.Block(ctx, blockRoot)
		if err != nil {
			return nil, err
		}
		summary := &ethpb.StateSummary{Slot: b.Block().Slot(), Root: blockRoot[:]}
		if err := s.beaconDB.SaveStateSummary(ctx, summary); err != nil {
			return nil, err
		}
		return summary, nil
	}
	return nil, errors.New("could not find block in DB")
}

// DeleteStateFromCaches deletes the state from the caches.
func (s *State) DeleteStateFromCaches(_ context.Context, blockRoot [32]byte) error {
	s.hotStateCache.delete(blockRoot)
	return s.epochBoundaryStateCache.delete(blockRoot)
}

// This loads a beacon state from either the cache or DB, then replays blocks up the slot of the requested block root.
func (s *State) loadStateByRoot(ctx context.Context, blockRoot [32]byte) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "stateGen.loadStateByRoot")
	defer span.End()

	// First, it checks if the state exists in hot state cache.
	cachedState := s.hotStateCache.get(blockRoot)
	if cachedState != nil && !cachedState.IsNil() {
		return cachedState, nil
	}

	// Second, it checks if the state exists in epoch boundary state cache.
	cachedInfo, ok, err := s.epochBoundaryStateCache.getByBlockRoot(blockRoot)
	if err != nil {
		return nil, err
	}
	if ok {
		return cachedInfo.state, nil
	}

	// Short circuit if the state is already in the DB.
	if s.beaconDB.HasState(ctx, blockRoot) {
		return s.beaconDB.State(ctx, blockRoot)
	}

	summary, err := s.stateSummary(ctx, blockRoot)
	if err != nil {
		return nil, errors.Wrap(err, "could not get state summary")
	}
	targetSlot := summary.Slot

	// Since the requested state is not in caches or DB, start replaying using the last
	// available ancestor state which is retrieved using input block's root.
	startState, err := s.latestAncestor(ctx, blockRoot)
	if err != nil {
		return nil, errors.Wrap(err, "could not get ancestor state")
	}
	if startState == nil || startState.IsNil() {
		return nil, errUnknownBoundaryState
	}

	if startState.Slot() == targetSlot {
		return startState, nil
	}

	blks, err := s.loadBlocks(ctx, startState.Slot()+1, targetSlot, bytesutil.ToBytes32(summary.Root))
	if err != nil {
		return nil, errors.Wrap(err, "could not load blocks for hot state using root")
	}

	replayBlockCount.Observe(float64(len(blks)))

	return s.replayBlocks(ctx, startState, blks, targetSlot)
}

// latestAncestor returns the highest available ancestor state of the input block root.
// It recursively looks up block's parent until a corresponding state of the block root
// is found in the caches or DB.
//
// There's three ways to derive block parent state:
// 1) block parent state is the last finalized state
// 2) block parent state is the epoch boundary state and exists in epoch boundary cache
// 3) block parent state is in DB
func (s *State) latestAncestor(ctx context.Context, blockRoot [32]byte) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "stateGen.latestAncestor")
	defer span.End()

	if s.isFinalizedRoot(blockRoot) {
		finalizedState := s.finalizedState()
		if finalizedState != nil {
			return finalizedState, nil
		}
	}

	b, err := s.beaconDB.Block(ctx, blockRoot)
	if err != nil {
		return nil, err
	}
	if err := blocks.BeaconBlockIsNil(b); err != nil {
		return nil, err
	}

	for {
		if ctx.Err() != nil {
			return nil, ctx.Err()
		}

		// Is the state the genesis state.
		parentRoot := b.Block().ParentRoot()
		if parentRoot == params.BeaconConfig().ZeroHash {
			s, err := s.beaconDB.GenesisState(ctx)
			return s, errors.Wrap(err, "could not get genesis state")
		}

		// Return an error if slot hasn't been covered by checkpoint sync.
		ps := b.Block().Slot() - 1
		if !s.slotAvailable(ps) {
			return nil, errors.Wrapf(ErrNoDataForSlot, "slot %d not in db due to checkpoint sync", ps)
		}
		// Does the state exist in the hot state cache.
		if s.hotStateCache.has(parentRoot) {
			return s.hotStateCache.get(parentRoot), nil
		}

		// Does the state exist in finalized info cache.
		if s.isFinalizedRoot(parentRoot) {
			return s.finalizedState(), nil
		}

		// Does the state exist in epoch boundary cache.
		cachedInfo, ok, err := s.epochBoundaryStateCache.getByBlockRoot(parentRoot)
		if err != nil {
			return nil, err
		}
		if ok {
			return cachedInfo.state, nil
		}

		// Does the state exists in DB.
		if s.beaconDB.HasState(ctx, parentRoot) {
			s, err := s.beaconDB.State(ctx, parentRoot)
			return s, errors.Wrap(err, "failed to retrieve state from db")
		}

		b, err = s.beaconDB.Block(ctx, parentRoot)
		if err != nil {
			return nil, errors.Wrap(err, "failed to retrieve block from db")
		}
		if b == nil || b.IsNil() {
			return nil, errUnknownBlock
		}
	}
}

func (s *State) CombinedCache() *CombinedCache {
	getters := make([]CachedGetter, 0)
	if s.hotStateCache != nil {
		getters = append(getters, s.hotStateCache)
	}
	if s.epochBoundaryStateCache != nil {
		getters = append(getters, s.epochBoundaryStateCache)
	}
	return &CombinedCache{getters: getters}
}

func (s *State) slotAvailable(slot primitives.Slot) bool {
	if s.avb == nil {
		return true
	}
	return s.avb.AvailableBlock(slot)
}
