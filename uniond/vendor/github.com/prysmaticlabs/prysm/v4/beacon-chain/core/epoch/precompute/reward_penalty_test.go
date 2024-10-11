package precompute

import (
	"context"
	"testing"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/math"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestProcessRewardsAndPenaltiesPrecompute(t *testing.T) {
	e := params.BeaconConfig().SlotsPerEpoch
	validatorCount := uint64(2048)
	base := buildState(e+3, validatorCount)
	atts := make([]*ethpb.PendingAttestation, 3)
	for i := 0; i < len(atts); i++ {
		atts[i] = &ethpb.PendingAttestation{
			Data: &ethpb.AttestationData{
				Target: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
				Source: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
			},
			AggregationBits: bitfield.Bitlist{0x00, 0x00, 0x00, 0x00, 0xC0, 0xC0, 0xC0, 0xC0, 0x01},
			InclusionDelay:  1,
		}
	}
	base.PreviousEpochAttestations = atts

	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)

	vp, bp, err := New(context.Background(), beaconState)
	require.NoError(t, err)
	vp, bp, err = ProcessAttestations(context.Background(), beaconState, vp, bp)
	require.NoError(t, err)

	processedState, err := ProcessRewardsAndPenaltiesPrecompute(beaconState, bp, vp, AttestationsDelta, ProposersDelta)
	require.NoError(t, err)
	require.Equal(t, true, processedState.Version() == version.Phase0)

	// Indices that voted everything except for head, lost a bit money
	wanted := uint64(31999810265)
	assert.Equal(t, wanted, beaconState.Balances()[4], "Unexpected balance")

	// Indices that did not vote, lost more money
	wanted = uint64(31999873505)
	assert.Equal(t, wanted, beaconState.Balances()[0], "Unexpected balance")
}

func TestAttestationDeltaPrecompute(t *testing.T) {
	e := params.BeaconConfig().SlotsPerEpoch
	validatorCount := uint64(2048)
	base := buildState(e+2, validatorCount)
	atts := make([]*ethpb.PendingAttestation, 3)
	var emptyRoot [32]byte
	for i := 0; i < len(atts); i++ {
		atts[i] = &ethpb.PendingAttestation{
			Data: &ethpb.AttestationData{
				Target: &ethpb.Checkpoint{
					Root: emptyRoot[:],
				},
				Source: &ethpb.Checkpoint{
					Root: emptyRoot[:],
				},
				BeaconBlockRoot: emptyRoot[:],
			},
			AggregationBits: bitfield.Bitlist{0xC0, 0xC0, 0xC0, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x01},
			InclusionDelay:  1,
		}
	}
	base.PreviousEpochAttestations = atts
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	slashedAttestedIndices := []primitives.ValidatorIndex{1413}
	for _, i := range slashedAttestedIndices {
		vs := beaconState.Validators()
		vs[i].Slashed = true
		require.Equal(t, nil, beaconState.SetValidators(vs))
	}

	vp, bp, err := New(context.Background(), beaconState)
	require.NoError(t, err)
	vp, bp, err = ProcessAttestations(context.Background(), beaconState, vp, bp)
	require.NoError(t, err)

	// Add some variances to target and head balances.
	// See: https://github.com/prysmaticlabs/prysm/issues/5593
	bp.PrevEpochTargetAttested /= 2
	bp.PrevEpochHeadAttested = bp.PrevEpochHeadAttested * 2 / 3
	rewards, penalties, err := AttestationsDelta(beaconState, bp, vp)
	require.NoError(t, err)
	attestedBalance, err := epoch.AttestingBalance(context.Background(), beaconState, atts)
	require.NoError(t, err)
	totalBalance, err := helpers.TotalActiveBalance(beaconState)
	require.NoError(t, err)

	attestedIndices := []primitives.ValidatorIndex{55, 1339, 1746, 1811, 1569}
	for _, i := range attestedIndices {
		base, err := baseReward(beaconState, i)
		require.NoError(t, err, "Could not get base reward")

		// Base rewards for getting source right
		wanted := attestedBalance*base/totalBalance +
			bp.PrevEpochTargetAttested*base/totalBalance +
			bp.PrevEpochHeadAttested*base/totalBalance
		// Base rewards for proposer and attesters working together getting attestation
		// on chain in the fatest manner
		proposerReward := base / params.BeaconConfig().ProposerRewardQuotient
		wanted += (base-proposerReward)*uint64(params.BeaconConfig().MinAttestationInclusionDelay) - 1
		assert.Equal(t, wanted, rewards[i], "Unexpected reward balance for validator with index %d", i)
		// Since all these validators attested, they shouldn't get penalized.
		assert.Equal(t, uint64(0), penalties[i], "Unexpected penalty balance")
	}

	for _, i := range slashedAttestedIndices {
		base, err := baseReward(beaconState, i)
		assert.NoError(t, err, "Could not get base reward")
		assert.Equal(t, uint64(0), rewards[i], "Unexpected slashed indices reward balance")
		assert.Equal(t, 3*base, penalties[i], "Unexpected slashed indices penalty balance")
	}

	nonAttestedIndices := []primitives.ValidatorIndex{434, 677, 872, 791}
	for _, i := range nonAttestedIndices {
		base, err := baseReward(beaconState, i)
		assert.NoError(t, err, "Could not get base reward")
		wanted := 3 * base
		// Since all these validators did not attest, they shouldn't get rewarded.
		assert.Equal(t, uint64(0), rewards[i], "Unexpected reward balance")
		// Base penalties for not attesting.
		assert.Equal(t, wanted, penalties[i], "Unexpected penalty balance")
	}
}

func TestAttestationDeltas_ZeroEpoch(t *testing.T) {
	e := params.BeaconConfig().SlotsPerEpoch
	validatorCount := uint64(2048)
	base := buildState(e+2, validatorCount)
	atts := make([]*ethpb.PendingAttestation, 3)
	var emptyRoot [32]byte
	for i := 0; i < len(atts); i++ {
		atts[i] = &ethpb.PendingAttestation{
			Data: &ethpb.AttestationData{
				Target: &ethpb.Checkpoint{
					Root: emptyRoot[:],
				},
				Source: &ethpb.Checkpoint{
					Root: emptyRoot[:],
				},
				BeaconBlockRoot: emptyRoot[:],
			},
			AggregationBits: bitfield.Bitlist{0x00, 0x00, 0x00, 0x00, 0xC0, 0xC0, 0xC0, 0xC0, 0x01},
			InclusionDelay:  1,
		}
	}
	base.PreviousEpochAttestations = atts
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)

	pVals, pBal, err := New(context.Background(), beaconState)
	assert.NoError(t, err)
	pVals, pBal, err = ProcessAttestations(context.Background(), beaconState, pVals, pBal)
	require.NoError(t, err)

	pBal.ActiveCurrentEpoch = 0 // Could cause a divide by zero panic.

	_, _, err = AttestationsDelta(beaconState, pBal, pVals)
	require.NoError(t, err)
}

func TestAttestationDeltas_ZeroInclusionDelay(t *testing.T) {
	e := params.BeaconConfig().SlotsPerEpoch
	validatorCount := uint64(2048)
	base := buildState(e+2, validatorCount)
	atts := make([]*ethpb.PendingAttestation, 3)
	var emptyRoot [32]byte
	for i := 0; i < len(atts); i++ {
		atts[i] = &ethpb.PendingAttestation{
			Data: &ethpb.AttestationData{
				Target: &ethpb.Checkpoint{
					Root: emptyRoot[:],
				},
				Source: &ethpb.Checkpoint{
					Root: emptyRoot[:],
				},
				BeaconBlockRoot: emptyRoot[:],
			},
			AggregationBits: bitfield.Bitlist{0xC0, 0xC0, 0xC0, 0xC0, 0x01},
			// Inclusion delay of 0 is not possible in a valid state and could cause a divide by
			// zero panic.
			InclusionDelay: 0,
		}
	}
	base.PreviousEpochAttestations = atts
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)

	pVals, pBal, err := New(context.Background(), beaconState)
	require.NoError(t, err)
	_, _, err = ProcessAttestations(context.Background(), beaconState, pVals, pBal)
	require.ErrorContains(t, "attestation with inclusion delay of 0", err)
}

func TestProcessRewardsAndPenaltiesPrecompute_SlashedInactivePenalty(t *testing.T) {
	e := params.BeaconConfig().SlotsPerEpoch
	validatorCount := uint64(2048)
	base := buildState(e+3, validatorCount)
	atts := make([]*ethpb.PendingAttestation, 3)
	for i := 0; i < len(atts); i++ {
		atts[i] = &ethpb.PendingAttestation{
			Data: &ethpb.AttestationData{
				Target: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
				Source: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
			},
			AggregationBits: bitfield.Bitlist{0x00, 0x00, 0x00, 0x00, 0xC0, 0xC0, 0xC0, 0xC0, 0x01},
			InclusionDelay:  1,
		}
	}
	base.PreviousEpochAttestations = atts

	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch*10))

	slashedAttestedIndices := []primitives.ValidatorIndex{14, 37, 68, 77, 139}
	for _, i := range slashedAttestedIndices {
		vs := beaconState.Validators()
		vs[i].Slashed = true
		require.NoError(t, beaconState.SetValidators(vs))
	}

	vp, bp, err := New(context.Background(), beaconState)
	require.NoError(t, err)
	vp, bp, err = ProcessAttestations(context.Background(), beaconState, vp, bp)
	require.NoError(t, err)
	rewards, penalties, err := AttestationsDelta(beaconState, bp, vp)
	require.NoError(t, err)

	finalityDelay := time.PrevEpoch(beaconState) - beaconState.FinalizedCheckpointEpoch()
	for _, i := range slashedAttestedIndices {
		base, err := baseReward(beaconState, i)
		require.NoError(t, err, "Could not get base reward")
		penalty := 3 * base
		proposerReward := base / params.BeaconConfig().ProposerRewardQuotient
		penalty += params.BeaconConfig().BaseRewardsPerEpoch*base - proposerReward
		penalty += vp[i].CurrentEpochEffectiveBalance * uint64(finalityDelay) / params.BeaconConfig().InactivityPenaltyQuotient
		assert.Equal(t, penalty, penalties[i], "Unexpected slashed indices penalty balance")
		assert.Equal(t, uint64(0), rewards[i], "Unexpected slashed indices reward balance")
	}
}

func buildState(slot primitives.Slot, validatorCount uint64) *ethpb.BeaconState {
	validators := make([]*ethpb.Validator, validatorCount)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance,
		}
	}
	validatorBalances := make([]uint64, len(validators))
	for i := 0; i < len(validatorBalances); i++ {
		validatorBalances[i] = params.BeaconConfig().MaxEffectiveBalance
	}
	latestActiveIndexRoots := make(
		[][]byte,
		params.BeaconConfig().EpochsPerHistoricalVector,
	)
	for i := 0; i < len(latestActiveIndexRoots); i++ {
		latestActiveIndexRoots[i] = params.BeaconConfig().ZeroHash[:]
	}
	latestRandaoMixes := make(
		[][]byte,
		params.BeaconConfig().EpochsPerHistoricalVector,
	)
	for i := 0; i < len(latestRandaoMixes); i++ {
		latestRandaoMixes[i] = params.BeaconConfig().ZeroHash[:]
	}
	return &ethpb.BeaconState{
		Slot:                        slot,
		Balances:                    validatorBalances,
		Validators:                  validators,
		RandaoMixes:                 make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
		Slashings:                   make([]uint64, params.BeaconConfig().EpochsPerSlashingsVector),
		BlockRoots:                  make([][]byte, params.BeaconConfig().SlotsPerEpoch*10),
		FinalizedCheckpoint:         &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
		PreviousJustifiedCheckpoint: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
		CurrentJustifiedCheckpoint:  &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
	}
}

func TestProposerDeltaPrecompute_HappyCase(t *testing.T) {
	e := params.BeaconConfig().SlotsPerEpoch
	validatorCount := uint64(10)
	base := buildState(e, validatorCount)
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)

	proposerIndex := primitives.ValidatorIndex(1)
	b := &Balance{ActiveCurrentEpoch: 1000}
	v := []*Validator{
		{IsPrevEpochAttester: true, CurrentEpochEffectiveBalance: 32, ProposerIndex: proposerIndex},
	}
	r, err := ProposersDelta(beaconState, b, v)
	require.NoError(t, err)

	baseReward := v[0].CurrentEpochEffectiveBalance * params.BeaconConfig().BaseRewardFactor /
		math.IntegerSquareRoot(b.ActiveCurrentEpoch) / params.BeaconConfig().BaseRewardsPerEpoch
	proposerReward := baseReward / params.BeaconConfig().ProposerRewardQuotient

	assert.Equal(t, proposerReward, r[proposerIndex], "Unexpected proposer reward")
}

func TestProposerDeltaPrecompute_ValidatorIndexOutOfRange(t *testing.T) {
	e := params.BeaconConfig().SlotsPerEpoch
	validatorCount := uint64(10)
	base := buildState(e, validatorCount)
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)

	proposerIndex := primitives.ValidatorIndex(validatorCount)
	b := &Balance{ActiveCurrentEpoch: 1000}
	v := []*Validator{
		{IsPrevEpochAttester: true, CurrentEpochEffectiveBalance: 32, ProposerIndex: proposerIndex},
	}
	_, err = ProposersDelta(beaconState, b, v)
	assert.ErrorContains(t, "proposer index out of range", err)
}

func TestProposerDeltaPrecompute_SlashedCase(t *testing.T) {
	e := params.BeaconConfig().SlotsPerEpoch
	validatorCount := uint64(10)
	base := buildState(e, validatorCount)
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)

	proposerIndex := primitives.ValidatorIndex(1)
	b := &Balance{ActiveCurrentEpoch: 1000}
	v := []*Validator{
		{IsPrevEpochAttester: true, CurrentEpochEffectiveBalance: 32, ProposerIndex: proposerIndex, IsSlashed: true},
	}
	r, err := ProposersDelta(beaconState, b, v)
	require.NoError(t, err)
	assert.Equal(t, uint64(0), r[proposerIndex], "Unexpected proposer reward for slashed")
}

// BaseReward takes state and validator index and calculate
// individual validator's base reward quotient.
//
// Spec pseudocode definition:
//
//	def get_base_reward(state: BeaconState, index: ValidatorIndex) -> Gwei:
//	  total_balance = get_total_active_balance(state)
//	  effective_balance = state.validators[index].effective_balance
//	  return Gwei(effective_balance * BASE_REWARD_FACTOR // integer_squareroot(total_balance) // BASE_REWARDS_PER_EPOCH)
func baseReward(state state.ReadOnlyBeaconState, index primitives.ValidatorIndex) (uint64, error) {
	totalBalance, err := helpers.TotalActiveBalance(state)
	if err != nil {
		return 0, errors.Wrap(err, "could not calculate active balance")
	}
	val, err := state.ValidatorAtIndexReadOnly(index)
	if err != nil {
		return 0, err
	}
	effectiveBalance := val.EffectiveBalance()
	baseReward := effectiveBalance * params.BeaconConfig().BaseRewardFactor /
		math.IntegerSquareRoot(totalBalance) / params.BeaconConfig().BaseRewardsPerEpoch
	return baseReward, nil
}
