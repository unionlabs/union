package helpers

import (
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"strconv"
	"strings"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/math"
	v1alpha1 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// ComputeWeakSubjectivityPeriod returns weak subjectivity period for the active validator count and finalized epoch.
//
// Reference spec implementation:
// https://github.com/ethereum/consensus-specs/blob/master/specs/phase0/weak-subjectivity.md#calculating-the-weak-subjectivity-period
//
// def compute_weak_subjectivity_period(state: BeaconState) -> uint64:
//
//	"""
//	Returns the weak subjectivity period for the current ``state``.
//	This computation takes into account the effect of:
//	    - validator set churn (bounded by ``get_validator_churn_limit()`` per epoch), and
//	    - validator balance top-ups (bounded by ``MAX_DEPOSITS * SLOTS_PER_EPOCH`` per epoch).
//	A detailed calculation can be found at:
//	https://github.com/runtimeverification/beacon-chain-verification/blob/master/weak-subjectivity/weak-subjectivity-analysis.pdf
//	"""
//	ws_period = MIN_VALIDATOR_WITHDRAWABILITY_DELAY
//	N = len(get_active_validator_indices(state, get_current_epoch(state)))
//	t = get_total_active_balance(state) // N // ETH_TO_GWEI
//	T = MAX_EFFECTIVE_BALANCE // ETH_TO_GWEI
//	delta = get_validator_churn_limit(state)
//	Delta = MAX_DEPOSITS * SLOTS_PER_EPOCH
//	D = SAFETY_DECAY
//
//	if T * (200 + 3 * D) < t * (200 + 12 * D):
//	    epochs_for_validator_set_churn = (
//	        N * (t * (200 + 12 * D) - T * (200 + 3 * D)) // (600 * delta * (2 * t + T))
//	    )
//	    epochs_for_balance_top_ups = (
//	        N * (200 + 3 * D) // (600 * Delta)
//	    )
//	    ws_period += max(epochs_for_validator_set_churn, epochs_for_balance_top_ups)
//	else:
//	    ws_period += (
//	        3 * N * D * t // (200 * Delta * (T - t))
//	    )
//
//	return ws_period
func ComputeWeakSubjectivityPeriod(ctx context.Context, st state.ReadOnlyBeaconState, cfg *params.BeaconChainConfig) (primitives.Epoch, error) {
	// Weak subjectivity period cannot be smaller than withdrawal delay.
	wsp := uint64(cfg.MinValidatorWithdrawabilityDelay)

	// Cardinality of active validator set.
	N, err := ActiveValidatorCount(ctx, st, time.CurrentEpoch(st))
	if err != nil {
		return 0, fmt.Errorf("cannot obtain active valiadtor count: %w", err)
	}
	if N == 0 {
		return 0, errors.New("no active validators found")
	}

	// Average effective balance in the given validator set, in Ether.
	t, err := TotalActiveBalance(st)
	if err != nil {
		return 0, fmt.Errorf("cannot find total active balance of validators: %w", err)
	}
	t = t / N / cfg.GweiPerEth

	// Maximum effective balance per validator.
	T := cfg.MaxEffectiveBalance / cfg.GweiPerEth

	// Validator churn limit.
	delta := ValidatorExitChurnLimit(N)

	// Balance top-ups.
	Delta := uint64(cfg.SlotsPerEpoch.Mul(cfg.MaxDeposits))

	if delta == 0 || Delta == 0 {
		return 0, errors.New("either validator churn limit or balance top-ups is zero")
	}

	// Safety decay, maximum tolerable loss of safety margin of FFG finality.
	D := cfg.SafetyDecay

	if T*(200+3*D) < t*(200+12*D) {
		epochsForValidatorSetChurn := N * (t*(200+12*D) - T*(200+3*D)) / (600 * delta * (2*t + T))
		epochsForBalanceTopUps := N * (200 + 3*D) / (600 * Delta)
		wsp += math.Max(epochsForValidatorSetChurn, epochsForBalanceTopUps)
	} else {
		wsp += 3 * N * D * t / (200 * Delta * (T - t))
	}

	return primitives.Epoch(wsp), nil
}

// IsWithinWeakSubjectivityPeriod verifies if a given weak subjectivity checkpoint is not stale i.e.
// the current node is so far beyond, that a given state and checkpoint are not for the latest weak
// subjectivity point. Provided checkpoint still can be used to double-check that node's block root
// at a given epoch matches that of the checkpoint.
//
// Reference implementation:
// https://github.com/ethereum/consensus-specs/blob/master/specs/phase0/weak-subjectivity.md#checking-for-stale-weak-subjectivity-checkpoint
//
// def is_within_weak_subjectivity_period(store: Store, ws_state: BeaconState, ws_checkpoint: Checkpoint) -> bool:
//
//	# Clients may choose to validate the input state against the input Weak Subjectivity Checkpoint
//	assert ws_state.latest_block_header.state_root == ws_checkpoint.root
//	assert compute_epoch_at_slot(ws_state.slot) == ws_checkpoint.epoch
//
//	ws_period = compute_weak_subjectivity_period(ws_state)
//	ws_state_epoch = compute_epoch_at_slot(ws_state.slot)
//	current_epoch = compute_epoch_at_slot(get_current_slot(store))
//	return current_epoch <= ws_state_epoch + ws_period
func IsWithinWeakSubjectivityPeriod(
	ctx context.Context, currentEpoch primitives.Epoch, wsState state.ReadOnlyBeaconState, wsStateRoot [fieldparams.RootLength]byte, wsEpoch primitives.Epoch, cfg *params.BeaconChainConfig) (bool, error) {
	// Make sure that incoming objects are not nil.
	if wsState == nil || wsState.IsNil() || wsState.LatestBlockHeader() == nil {
		return false, errors.New("invalid weak subjectivity state or checkpoint")
	}

	// Assert that state and checkpoint have the same root and epoch.
	if bytesutil.ToBytes32(wsState.LatestBlockHeader().StateRoot) != wsStateRoot {
		return false, fmt.Errorf("state (%#x) and checkpoint (%#x) roots do not match",
			wsState.LatestBlockHeader().StateRoot, wsStateRoot)
	}
	if slots.ToEpoch(wsState.Slot()) != wsEpoch {
		return false, fmt.Errorf("state (%v) and checkpoint (%v) epochs do not match",
			slots.ToEpoch(wsState.Slot()), wsEpoch)
	}

	// Compare given epoch to state epoch + weak subjectivity period.
	wsPeriod, err := ComputeWeakSubjectivityPeriod(ctx, wsState, cfg)
	if err != nil {
		return false, fmt.Errorf("cannot compute weak subjectivity period: %w", err)
	}
	wsStateEpoch := slots.ToEpoch(wsState.Slot())

	return currentEpoch <= wsStateEpoch+wsPeriod, nil
}

// LatestWeakSubjectivityEpoch returns epoch of the most recent weak subjectivity checkpoint known to a node.
//
// Within the weak subjectivity period, if two conflicting blocks are finalized, 1/3 - D (D := safety decay)
// of validators will get slashed. Therefore, it is safe to assume that any finalized checkpoint within that
// period is protected by this safety margin.
func LatestWeakSubjectivityEpoch(ctx context.Context, st state.ReadOnlyBeaconState, cfg *params.BeaconChainConfig) (primitives.Epoch, error) {
	wsPeriod, err := ComputeWeakSubjectivityPeriod(ctx, st, cfg)
	if err != nil {
		return 0, err
	}

	finalizedEpoch := st.FinalizedCheckpointEpoch()
	return finalizedEpoch - (finalizedEpoch % wsPeriod), nil
}

// ParseWeakSubjectivityInputString parses "blocks_root:epoch_number" string into a checkpoint.
func ParseWeakSubjectivityInputString(wsCheckpointString string) (*v1alpha1.Checkpoint, error) {
	if wsCheckpointString == "" {
		return nil, nil
	}

	// Weak subjectivity input string must contain ":" to separate epoch and block root.
	if !strings.Contains(wsCheckpointString, ":") {
		return nil, fmt.Errorf("%s did not contain column", wsCheckpointString)
	}

	// Strip prefix "0x" if it's part of the input string.
	wsCheckpointString = strings.TrimPrefix(wsCheckpointString, "0x")

	// Get the hexadecimal block root from input string.
	s := strings.Split(wsCheckpointString, ":")
	if len(s) != 2 {
		return nil, errors.New("weak subjectivity checkpoint input should be in `block_root:epoch_number` format")
	}

	bRoot, err := hex.DecodeString(s[0])
	if err != nil {
		return nil, err
	}
	if len(bRoot) != 32 {
		return nil, errors.New("block root is not length of 32")
	}

	// Get the epoch number from input string.
	epoch, err := strconv.ParseUint(s[1], 10, 64)
	if err != nil {
		return nil, err
	}

	return &v1alpha1.Checkpoint{
		Epoch: primitives.Epoch(epoch),
		Root:  bRoot,
	}, nil
}

// MinEpochsForBlockRequests computes the number of epochs of block history that we need to maintain,
// relative to the current epoch, per the p2p specs. This is used to compute the slot where backfill is complete.
// value defined:
// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/p2p-interface.md#configuration
// MIN_VALIDATOR_WITHDRAWABILITY_DELAY + CHURN_LIMIT_QUOTIENT // 2 (= 33024, ~5 months)
// detailed rationale: https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/p2p-interface.md#why-are-blocksbyrange-requests-only-required-to-be-served-for-the-latest-min_epochs_for_block_requests-epochs
func MinEpochsForBlockRequests() primitives.Epoch {
	return params.BeaconConfig().MinValidatorWithdrawabilityDelay +
		primitives.Epoch(params.BeaconConfig().ChurnLimitQuotient/2)
}
