// Package transition implements the whole state transition
// function which consists of per slot, per-epoch transitions.
// It also bootstraps the genesis beacon state for slot 0.
package transition

import (
	"bytes"
	"context"
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/capella"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/deneb"
	e "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch/precompute"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/execution"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/math"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"go.opencensus.io/trace"
)

// ExecuteStateTransition defines the procedure for a state transition function.
//
// Note: This method differs from the spec pseudocode as it uses a batch signature verification.
// See: ExecuteStateTransitionNoVerifyAnySig
//
// Spec pseudocode definition:
//
//	def state_transition(state: BeaconState, signed_block: ReadOnlySignedBeaconBlock, validate_result: bool=True) -> None:
//	  block = signed_block.message
//	  # Process slots (including those with no blocks) since block
//	  process_slots(state, block.slot)
//	  # Verify signature
//	  if validate_result:
//	      assert verify_block_signature(state, signed_block)
//	  # Process block
//	  process_block(state, block)
//	  # Verify state root
//	  if validate_result:
//	      assert block.state_root == hash_tree_root(state)
func ExecuteStateTransition(
	ctx context.Context,
	state state.BeaconState,
	signed interfaces.ReadOnlySignedBeaconBlock,
) (state.BeaconState, error) {
	if ctx.Err() != nil {
		return nil, ctx.Err()
	}
	if err := blocks.BeaconBlockIsNil(signed); err != nil {
		return nil, err
	}

	ctx, span := trace.StartSpan(ctx, "core.state.ExecuteStateTransition")
	defer span.End()
	var err error

	set, postState, err := ExecuteStateTransitionNoVerifyAnySig(ctx, state, signed)
	if err != nil {
		return nil, errors.Wrap(err, "could not execute state transition")
	}

	var valid bool
	if features.Get().EnableVerboseSigVerification {
		valid, err = set.VerifyVerbosely()
	} else {
		valid, err = set.Verify()
	}
	if err != nil {
		return nil, errors.Wrap(err, "could not batch verify signature")
	}
	if !valid {
		return nil, errors.New("signature in block failed to verify")
	}

	return postState, nil
}

// ProcessSlot happens every slot and focuses on the slot counter and block roots record updates.
// It happens regardless if there's an incoming block or not.
// Spec pseudocode definition:
//
//	def process_slot(state: BeaconState) -> None:
//	  # Cache state root
//	  previous_state_root = hash_tree_root(state)
//	  state.state_roots[state.slot % SLOTS_PER_HISTORICAL_ROOT] = previous_state_root
//	  # Cache latest block header state root
//	  if state.latest_block_header.state_root == Bytes32():
//	      state.latest_block_header.state_root = previous_state_root
//	  # Cache block root
//	  previous_block_root = hash_tree_root(state.latest_block_header)
//	  state.block_roots[state.slot % SLOTS_PER_HISTORICAL_ROOT] = previous_block_root
func ProcessSlot(ctx context.Context, state state.BeaconState) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "core.state.ProcessSlot")
	defer span.End()
	span.AddAttributes(trace.Int64Attribute("slot", int64(state.Slot()))) // lint:ignore uintcast -- This is OK for tracing.

	prevStateRoot, err := state.HashTreeRoot(ctx)
	if err != nil {
		return nil, err
	}
	if err := state.UpdateStateRootAtIndex(
		uint64(state.Slot()%params.BeaconConfig().SlotsPerHistoricalRoot),
		prevStateRoot,
	); err != nil {
		return nil, err
	}

	zeroHash := params.BeaconConfig().ZeroHash
	// Cache latest block header state root.
	header := state.LatestBlockHeader()
	if header.StateRoot == nil || bytes.Equal(header.StateRoot, zeroHash[:]) {
		header.StateRoot = prevStateRoot[:]
		if err := state.SetLatestBlockHeader(header); err != nil {
			return nil, err
		}
	}
	prevBlockRoot, err := state.LatestBlockHeader().HashTreeRoot()
	if err != nil {
		tracing.AnnotateError(span, err)
		return nil, errors.Wrap(err, "could not determine prev block root")
	}
	// Cache the block root.
	if err := state.UpdateBlockRootAtIndex(
		uint64(state.Slot()%params.BeaconConfig().SlotsPerHistoricalRoot),
		prevBlockRoot,
	); err != nil {
		return nil, err
	}
	return state, nil
}

// ProcessSlotsUsingNextSlotCache processes slots by using next slot cache for higher efficiency.
func ProcessSlotsUsingNextSlotCache(
	ctx context.Context,
	parentState state.BeaconState,
	parentRoot []byte,
	slot primitives.Slot) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "core.state.ProcessSlotsUsingNextSlotCache")
	defer span.End()

	nextSlotState := NextSlotState(parentRoot, slot)
	if nextSlotState != nil {
		parentState = nextSlotState
	}
	if parentState.Slot() == slot {
		return parentState, nil
	}

	var err error
	parentState, err = ProcessSlots(ctx, parentState, slot)
	if err != nil {
		return nil, errors.Wrap(err, "could not process slots")
	}
	return parentState, nil
}

// ProcessSlotsIfPossible executes ProcessSlots on the input state when target slot is above the state's slot.
// Otherwise, it returns the input state unchanged.
func ProcessSlotsIfPossible(ctx context.Context, state state.BeaconState, targetSlot primitives.Slot) (state.BeaconState, error) {
	if targetSlot > state.Slot() {
		return ProcessSlots(ctx, state, targetSlot)
	}
	return state, nil
}

// ProcessSlots process through skip slots and apply epoch transition when it's needed
//
// Spec pseudocode definition:
//
//	def process_slots(state: BeaconState, slot: Slot) -> None:
//	  assert state.slot < slot
//	  while state.slot < slot:
//	      process_slot(state)
//	      # Process epoch on the start slot of the next epoch
//	      if (state.slot + 1) % SLOTS_PER_EPOCH == 0:
//	          process_epoch(state)
//	      state.slot = Slot(state.slot + 1)
func ProcessSlots(ctx context.Context, state state.BeaconState, slot primitives.Slot) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "core.state.ProcessSlots")
	defer span.End()
	if state == nil || state.IsNil() {
		return nil, errors.New("nil state")
	}
	span.AddAttributes(trace.Int64Attribute("slots", int64(slot)-int64(state.Slot()))) // lint:ignore uintcast -- This is OK for tracing.

	// The block must have a higher slot than parent state.
	if state.Slot() >= slot {
		err := fmt.Errorf("expected state.slot %d < slot %d", state.Slot(), slot)
		tracing.AnnotateError(span, err)
		return nil, err
	}

	highestSlot := state.Slot()
	key, err := cacheKey(ctx, state)
	if err != nil {
		return nil, err
	}

	// Restart from cached value, if one exists.
	cachedState, err := SkipSlotCache.Get(ctx, key)
	if err != nil {
		return nil, err
	}

	if cachedState != nil && !cachedState.IsNil() && cachedState.Slot() < slot {
		highestSlot = cachedState.Slot()
		state = cachedState
	}
	if err := SkipSlotCache.MarkInProgress(key); errors.Is(err, cache.ErrAlreadyInProgress) {
		cachedState, err = SkipSlotCache.Get(ctx, key)
		if err != nil {
			return nil, err
		}
		if cachedState != nil && !cachedState.IsNil() && cachedState.Slot() < slot {
			highestSlot = cachedState.Slot()
			state = cachedState
		}
	} else if err != nil {
		return nil, err
	}
	defer func() {
		SkipSlotCache.MarkNotInProgress(key)
	}()

	for state.Slot() < slot {
		if ctx.Err() != nil {
			tracing.AnnotateError(span, ctx.Err())
			// Cache last best value.
			if highestSlot < state.Slot() {
				if SkipSlotCache.Put(ctx, key, state); err != nil {
					log.WithError(err).Error("Failed to put skip slot cache value")
				}
			}
			return nil, ctx.Err()
		}
		state, err = ProcessSlot(ctx, state)
		if err != nil {
			tracing.AnnotateError(span, err)
			return nil, errors.Wrap(err, "could not process slot")
		}
		if time.CanProcessEpoch(state) {
			if state.Version() == version.Phase0 {
				state, err = ProcessEpochPrecompute(ctx, state)
				if err != nil {
					tracing.AnnotateError(span, err)
					return nil, errors.Wrap(err, "could not process epoch with optimizations")
				}
			} else if state.Version() >= version.Altair {
				state, err = altair.ProcessEpoch(ctx, state)
				if err != nil {
					tracing.AnnotateError(span, err)
					return nil, errors.Wrap(err, "could not process epoch")
				}
			} else {
				return nil, errors.New("beacon state should have a version")
			}
		}
		if err := state.SetSlot(state.Slot() + 1); err != nil {
			tracing.AnnotateError(span, err)
			return nil, errors.Wrap(err, "failed to increment state slot")
		}

		state, err = UpgradeState(ctx, state)
		if err != nil {
			tracing.AnnotateError(span, err)
			return nil, errors.Wrap(err, "failed to upgrade state")
		}
	}

	if highestSlot < state.Slot() {
		SkipSlotCache.Put(ctx, key, state)
	}

	return state, nil
}

// UpgradeState upgrades the state to the next version if possible.
func UpgradeState(ctx context.Context, state state.BeaconState) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "core.state.UpgradeState")
	defer span.End()
	var err error
	if time.CanUpgradeToAltair(state.Slot()) {
		state, err = altair.UpgradeToAltair(ctx, state)
		if err != nil {
			tracing.AnnotateError(span, err)
			return nil, err
		}
	}

	if time.CanUpgradeToBellatrix(state.Slot()) {
		state, err = execution.UpgradeToBellatrix(state)
		if err != nil {
			tracing.AnnotateError(span, err)
			return nil, err
		}
	}

	if time.CanUpgradeToCapella(state.Slot()) {
		state, err = capella.UpgradeToCapella(state)
		if err != nil {
			tracing.AnnotateError(span, err)
			return nil, err
		}
	}

	if time.CanUpgradeToDeneb(state.Slot()) {
		state, err = deneb.UpgradeToDeneb(state)
		if err != nil {
			tracing.AnnotateError(span, err)
			return nil, err
		}
	}
	return state, nil
}

// VerifyOperationLengths verifies that block operation lengths are valid.
func VerifyOperationLengths(_ context.Context, state state.BeaconState, b interfaces.ReadOnlyBeaconBlock) (state.BeaconState, error) {
	if b == nil || b.IsNil() {
		return nil, blocks.ErrNilBeaconBlock
	}
	body := b.Body()

	if uint64(len(body.ProposerSlashings())) > params.BeaconConfig().MaxProposerSlashings {
		return nil, fmt.Errorf(
			"number of proposer slashings (%d) in block body exceeds allowed threshold of %d",
			len(body.ProposerSlashings()),
			params.BeaconConfig().MaxProposerSlashings,
		)
	}

	if uint64(len(body.AttesterSlashings())) > params.BeaconConfig().MaxAttesterSlashings {
		return nil, fmt.Errorf(
			"number of attester slashings (%d) in block body exceeds allowed threshold of %d",
			len(body.AttesterSlashings()),
			params.BeaconConfig().MaxAttesterSlashings,
		)
	}

	if uint64(len(body.Attestations())) > params.BeaconConfig().MaxAttestations {
		return nil, fmt.Errorf(
			"number of attestations (%d) in block body exceeds allowed threshold of %d",
			len(body.Attestations()),
			params.BeaconConfig().MaxAttestations,
		)
	}

	if uint64(len(body.VoluntaryExits())) > params.BeaconConfig().MaxVoluntaryExits {
		return nil, fmt.Errorf(
			"number of voluntary exits (%d) in block body exceeds allowed threshold of %d",
			len(body.VoluntaryExits()),
			params.BeaconConfig().MaxVoluntaryExits,
		)
	}
	eth1Data := state.Eth1Data()
	if eth1Data == nil {
		return nil, errors.New("nil eth1data in state")
	}
	if state.Eth1DepositIndex() > eth1Data.DepositCount {
		return nil, fmt.Errorf("expected state.deposit_index %d <= eth1data.deposit_count %d", state.Eth1DepositIndex(), eth1Data.DepositCount)
	}
	maxDeposits := math.Min(params.BeaconConfig().MaxDeposits, eth1Data.DepositCount-state.Eth1DepositIndex())
	// Verify outstanding deposits are processed up to max number of deposits
	if uint64(len(body.Deposits())) != maxDeposits {
		return nil, fmt.Errorf("incorrect outstanding deposits in block body, wanted: %d, got: %d",
			maxDeposits, len(body.Deposits()))
	}

	return state, nil
}

// ProcessEpochPrecompute describes the per epoch operations that are performed on the beacon state.
// It's optimized by pre computing validator attested info and epoch total/attested balances upfront.
func ProcessEpochPrecompute(ctx context.Context, state state.BeaconState) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "core.state.ProcessEpochPrecompute")
	defer span.End()
	span.AddAttributes(trace.Int64Attribute("epoch", int64(time.CurrentEpoch(state)))) // lint:ignore uintcast -- This is OK for tracing.

	if state == nil || state.IsNil() {
		return nil, errors.New("nil state")
	}
	vp, bp, err := precompute.New(ctx, state)
	if err != nil {
		return nil, err
	}
	vp, bp, err = precompute.ProcessAttestations(ctx, state, vp, bp)
	if err != nil {
		return nil, err
	}

	state, err = precompute.ProcessJustificationAndFinalizationPreCompute(state, bp)
	if err != nil {
		return nil, errors.Wrap(err, "could not process justification")
	}

	state, err = precompute.ProcessRewardsAndPenaltiesPrecompute(state, bp, vp, precompute.AttestationsDelta, precompute.ProposersDelta)
	if err != nil {
		return nil, errors.Wrap(err, "could not process rewards and penalties")
	}

	state, err = e.ProcessRegistryUpdates(ctx, state)
	if err != nil {
		return nil, errors.Wrap(err, "could not process registry updates")
	}

	err = precompute.ProcessSlashingsPrecompute(state, bp)
	if err != nil {
		return nil, err
	}

	state, err = e.ProcessFinalUpdates(state)
	if err != nil {
		return nil, errors.Wrap(err, "could not process final updates")
	}
	return state, nil
}
