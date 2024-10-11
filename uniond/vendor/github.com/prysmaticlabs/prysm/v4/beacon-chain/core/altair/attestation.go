package altair

import (
	"bytes"
	"context"
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation"
	"go.opencensus.io/trace"
)

// ProcessAttestationsNoVerifySignature applies processing operations to a block's inner attestation
// records. The only difference would be that the attestation signature would not be verified.
func ProcessAttestationsNoVerifySignature(
	ctx context.Context,
	beaconState state.BeaconState,
	b interfaces.ReadOnlyBeaconBlock,
) (state.BeaconState, error) {
	if b == nil || b.IsNil() {
		return nil, consensusblocks.ErrNilBeaconBlock
	}
	body := b.Body()
	totalBalance, err := helpers.TotalActiveBalance(beaconState)
	if err != nil {
		return nil, err
	}
	for idx, att := range body.Attestations() {
		beaconState, err = ProcessAttestationNoVerifySignature(ctx, beaconState, att, totalBalance)
		if err != nil {
			return nil, errors.Wrapf(err, "could not verify attestation at index %d in block", idx)
		}
	}
	return beaconState, nil
}

// ProcessAttestationNoVerifySignature processes the attestation without verifying the attestation signature. This
// method is used to validate attestations whose signatures have already been verified or will be verified later.
func ProcessAttestationNoVerifySignature(
	ctx context.Context,
	beaconState state.BeaconState,
	att *ethpb.Attestation,
	totalBalance uint64,
) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "altair.ProcessAttestationNoVerifySignature")
	defer span.End()

	if err := blocks.VerifyAttestationNoVerifySignature(ctx, beaconState, att); err != nil {
		return nil, err
	}

	delay, err := beaconState.Slot().SafeSubSlot(att.Data.Slot)
	if err != nil {
		return nil, fmt.Errorf("att slot %d can't be greater than state slot %d", att.Data.Slot, beaconState.Slot())
	}
	participatedFlags, err := AttestationParticipationFlagIndices(beaconState, att.Data, delay)
	if err != nil {
		return nil, err
	}
	committee, err := helpers.BeaconCommitteeFromState(ctx, beaconState, att.Data.Slot, att.Data.CommitteeIndex)
	if err != nil {
		return nil, err
	}
	indices, err := attestation.AttestingIndices(att.AggregationBits, committee)
	if err != nil {
		return nil, err
	}

	return SetParticipationAndRewardProposer(ctx, beaconState, att.Data.Target.Epoch, indices, participatedFlags, totalBalance)
}

// SetParticipationAndRewardProposer retrieves and sets the epoch participation bits in state. Based on the epoch participation, it rewards
// the proposer in state.
//
// Spec code:
//
//	 # Update epoch participation flags
//	if data.target.epoch == get_current_epoch(state):
//	    epoch_participation = state.current_epoch_participation
//	else:
//	    epoch_participation = state.previous_epoch_participation
//
//	proposer_reward_numerator = 0
//	for index in get_attesting_indices(state, data, attestation.aggregation_bits):
//	    for flag_index, weight in enumerate(PARTICIPATION_FLAG_WEIGHTS):
//	        if flag_index in participation_flag_indices and not has_flag(epoch_participation[index], flag_index):
//	            epoch_participation[index] = add_flag(epoch_participation[index], flag_index)
//	            proposer_reward_numerator += get_base_reward(state, index) * weight
//
//	# Reward proposer
//	proposer_reward_denominator = (WEIGHT_DENOMINATOR - PROPOSER_WEIGHT) * WEIGHT_DENOMINATOR // PROPOSER_WEIGHT
//	proposer_reward = Gwei(proposer_reward_numerator // proposer_reward_denominator)
//	increase_balance(state, get_beacon_proposer_index(state), proposer_reward)
func SetParticipationAndRewardProposer(
	ctx context.Context,
	beaconState state.BeaconState,
	targetEpoch primitives.Epoch,
	indices []uint64,
	participatedFlags map[uint8]bool, totalBalance uint64) (state.BeaconState, error) {
	var proposerRewardNumerator uint64
	currentEpoch := time.CurrentEpoch(beaconState)
	var stateErr error
	if targetEpoch == currentEpoch {
		stateErr = beaconState.ModifyCurrentParticipationBits(func(val []byte) ([]byte, error) {
			propRewardNum, epochParticipation, err := EpochParticipation(beaconState, indices, val, participatedFlags, totalBalance)
			if err != nil {
				return nil, err
			}
			proposerRewardNumerator = propRewardNum
			return epochParticipation, nil
		})
	} else {
		stateErr = beaconState.ModifyPreviousParticipationBits(func(val []byte) ([]byte, error) {
			propRewardNum, epochParticipation, err := EpochParticipation(beaconState, indices, val, participatedFlags, totalBalance)
			if err != nil {
				return nil, err
			}
			proposerRewardNumerator = propRewardNum
			return epochParticipation, nil
		})
	}
	if stateErr != nil {
		return nil, stateErr
	}

	if err := RewardProposer(ctx, beaconState, proposerRewardNumerator); err != nil {
		return nil, err
	}

	return beaconState, nil
}

// HasValidatorFlag returns true if the flag at position has set.
func HasValidatorFlag(flag, flagPosition uint8) (bool, error) {
	if flagPosition > 7 {
		return false, errors.New("flag position exceeds length")
	}
	return ((flag >> flagPosition) & 1) == 1, nil
}

// AddValidatorFlag adds new validator flag to existing one.
func AddValidatorFlag(flag, flagPosition uint8) (uint8, error) {
	if flagPosition > 7 {
		return flag, errors.New("flag position exceeds length")
	}
	return flag | (1 << flagPosition), nil
}

// EpochParticipation sets and returns the proposer reward numerator and epoch participation.
//
// Spec code:
//
//	proposer_reward_numerator = 0
//	for index in get_attesting_indices(state, data, attestation.aggregation_bits):
//	    for flag_index, weight in enumerate(PARTICIPATION_FLAG_WEIGHTS):
//	        if flag_index in participation_flag_indices and not has_flag(epoch_participation[index], flag_index):
//	            epoch_participation[index] = add_flag(epoch_participation[index], flag_index)
//	            proposer_reward_numerator += get_base_reward(state, index) * weight
func EpochParticipation(beaconState state.BeaconState, indices []uint64, epochParticipation []byte, participatedFlags map[uint8]bool, totalBalance uint64) (uint64, []byte, error) {
	cfg := params.BeaconConfig()
	sourceFlagIndex := cfg.TimelySourceFlagIndex
	targetFlagIndex := cfg.TimelyTargetFlagIndex
	headFlagIndex := cfg.TimelyHeadFlagIndex
	proposerRewardNumerator := uint64(0)
	for _, index := range indices {
		if index >= uint64(len(epochParticipation)) {
			return 0, nil, fmt.Errorf("index %d exceeds participation length %d", index, len(epochParticipation))
		}
		br, err := BaseRewardWithTotalBalance(beaconState, primitives.ValidatorIndex(index), totalBalance)
		if err != nil {
			return 0, nil, err
		}
		has, err := HasValidatorFlag(epochParticipation[index], sourceFlagIndex)
		if err != nil {
			return 0, nil, err
		}
		if participatedFlags[sourceFlagIndex] && !has {
			epochParticipation[index], err = AddValidatorFlag(epochParticipation[index], sourceFlagIndex)
			if err != nil {
				return 0, nil, err
			}
			proposerRewardNumerator += br * cfg.TimelySourceWeight
		}
		has, err = HasValidatorFlag(epochParticipation[index], targetFlagIndex)
		if err != nil {
			return 0, nil, err
		}
		if participatedFlags[targetFlagIndex] && !has {
			epochParticipation[index], err = AddValidatorFlag(epochParticipation[index], targetFlagIndex)
			if err != nil {
				return 0, nil, err
			}
			proposerRewardNumerator += br * cfg.TimelyTargetWeight
		}
		has, err = HasValidatorFlag(epochParticipation[index], headFlagIndex)
		if err != nil {
			return 0, nil, err
		}
		if participatedFlags[headFlagIndex] && !has {
			epochParticipation[index], err = AddValidatorFlag(epochParticipation[index], headFlagIndex)
			if err != nil {
				return 0, nil, err
			}
			proposerRewardNumerator += br * cfg.TimelyHeadWeight
		}
	}

	return proposerRewardNumerator, epochParticipation, nil
}

// RewardProposer rewards proposer by increasing proposer's balance with input reward numerator and calculated reward denominator.
//
// Spec code:
//
//	proposer_reward_denominator = (WEIGHT_DENOMINATOR - PROPOSER_WEIGHT) * WEIGHT_DENOMINATOR // PROPOSER_WEIGHT
//	proposer_reward = Gwei(proposer_reward_numerator // proposer_reward_denominator)
//	increase_balance(state, get_beacon_proposer_index(state), proposer_reward)
func RewardProposer(ctx context.Context, beaconState state.BeaconState, proposerRewardNumerator uint64) error {
	cfg := params.BeaconConfig()
	d := (cfg.WeightDenominator - cfg.ProposerWeight) * cfg.WeightDenominator / cfg.ProposerWeight
	proposerReward := proposerRewardNumerator / d
	i, err := helpers.BeaconProposerIndex(ctx, beaconState)
	if err != nil {
		return err
	}

	return helpers.IncreaseBalance(beaconState, i, proposerReward)
}

// AttestationParticipationFlagIndices retrieves a map of attestation scoring based on Altair's participation flag indices.
// This is used to facilitate process attestation during state transition and during upgrade to altair state.
//
// Spec code:
// def get_attestation_participation_flag_indices(state: BeaconState,
//
//	                                           data: AttestationData,
//	                                           inclusion_delay: uint64) -> Sequence[int]:
//	"""
//	Return the flag indices that are satisfied by an attestation.
//	"""
//	if data.target.epoch == get_current_epoch(state):
//	    justified_checkpoint = state.current_justified_checkpoint
//	else:
//	    justified_checkpoint = state.previous_justified_checkpoint
//
//	# Matching roots
//	is_matching_source = data.source == justified_checkpoint
//	is_matching_target = is_matching_source and data.target.root == get_block_root(state, data.target.epoch)
//	is_matching_head = is_matching_target and data.beacon_block_root == get_block_root_at_slot(state, data.slot)
//	assert is_matching_source
//
//	participation_flag_indices = []
//	if is_matching_source and inclusion_delay <= integer_squareroot(SLOTS_PER_EPOCH):
//	    participation_flag_indices.append(TIMELY_SOURCE_FLAG_INDEX)
//	if is_matching_target and inclusion_delay <= SLOTS_PER_EPOCH:
//	    participation_flag_indices.append(TIMELY_TARGET_FLAG_INDEX)
//	if is_matching_head and inclusion_delay == MIN_ATTESTATION_INCLUSION_DELAY:
//	    participation_flag_indices.append(TIMELY_HEAD_FLAG_INDEX)
//
//	return participation_flag_indices
func AttestationParticipationFlagIndices(beaconState state.BeaconState, data *ethpb.AttestationData, delay primitives.Slot) (map[uint8]bool, error) {
	currEpoch := time.CurrentEpoch(beaconState)
	var justifiedCheckpt *ethpb.Checkpoint
	if data.Target.Epoch == currEpoch {
		justifiedCheckpt = beaconState.CurrentJustifiedCheckpoint()
	} else {
		justifiedCheckpt = beaconState.PreviousJustifiedCheckpoint()
	}

	matchedSrc, matchedTgt, matchedHead, err := MatchingStatus(beaconState, data, justifiedCheckpt)
	if err != nil {
		return nil, err
	}
	if !matchedSrc {
		return nil, errors.New("source epoch does not match")
	}

	participatedFlags := make(map[uint8]bool)
	cfg := params.BeaconConfig()
	sourceFlagIndex := cfg.TimelySourceFlagIndex
	targetFlagIndex := cfg.TimelyTargetFlagIndex
	headFlagIndex := cfg.TimelyHeadFlagIndex
	sqtRootSlots := cfg.SqrRootSlotsPerEpoch
	if matchedSrc && delay <= sqtRootSlots {
		participatedFlags[sourceFlagIndex] = true
	}
	matchedSrcTgt := matchedSrc && matchedTgt
	// Before Deneb no attestation should pass validation without having delay <= slotsPerEpoch.
	if matchedSrcTgt {
		participatedFlags[targetFlagIndex] = true
	}
	matchedSrcTgtHead := matchedHead && matchedSrcTgt
	if matchedSrcTgtHead && delay == cfg.MinAttestationInclusionDelay {
		participatedFlags[headFlagIndex] = true
	}
	return participatedFlags, nil
}

// MatchingStatus returns the matching statues for attestation data's source target and head.
//
// Spec code:
//
//	is_matching_source = data.source == justified_checkpoint
//	is_matching_target = is_matching_source and data.target.root == get_block_root(state, data.target.epoch)
//	is_matching_head = is_matching_target and data.beacon_block_root == get_block_root_at_slot(state, data.slot)
func MatchingStatus(beaconState state.BeaconState, data *ethpb.AttestationData, cp *ethpb.Checkpoint) (matchedSrc, matchedTgt, matchedHead bool, err error) {
	matchedSrc = attestation.CheckPointIsEqual(data.Source, cp)

	r, err := helpers.BlockRoot(beaconState, data.Target.Epoch)
	if err != nil {
		return false, false, false, err
	}
	matchedTgt = bytes.Equal(r, data.Target.Root)

	r, err = helpers.BlockRootAtSlot(beaconState, data.Slot)
	if err != nil {
		return false, false, false, err
	}
	matchedHead = bytes.Equal(r, data.BeaconBlockRoot)
	return
}
