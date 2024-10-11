package epoch_test

import (
	"context"
	"fmt"
	"math"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/proto"
)

func TestUnslashedAttestingIndices_CanSortAndFilter(t *testing.T) {
	// Generate 2 attestations.
	atts := make([]*ethpb.PendingAttestation, 2)
	for i := 0; i < len(atts); i++ {
		atts[i] = &ethpb.PendingAttestation{
			Data: &ethpb.AttestationData{Source: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
				Target: &ethpb.Checkpoint{Epoch: 0, Root: make([]byte, fieldparams.RootLength)},
			},
			AggregationBits: bitfield.Bitlist{0x00, 0xFF, 0xFF, 0xFF},
		}
	}

	// Generate validators and state for the 2 attestations.
	validatorCount := 1000
	validators := make([]*ethpb.Validator, validatorCount)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}
	base := &ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	}
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)

	indices, err := epoch.UnslashedAttestingIndices(context.Background(), beaconState, atts)
	require.NoError(t, err)
	for i := 0; i < len(indices)-1; i++ {
		if indices[i] >= indices[i+1] {
			t.Error("sorted indices not sorted or duplicated")
		}
	}

	// Verify the slashed validator is filtered.
	slashedValidator := indices[0]
	validators = beaconState.Validators()
	validators[slashedValidator].Slashed = true
	require.NoError(t, beaconState.SetValidators(validators))
	indices, err = epoch.UnslashedAttestingIndices(context.Background(), beaconState, atts)
	require.NoError(t, err)
	for i := 0; i < len(indices); i++ {
		assert.NotEqual(t, slashedValidator, indices[i], "Slashed validator %d is not filtered", slashedValidator)
	}
}

func TestUnslashedAttestingIndices_DuplicatedAttestations(t *testing.T) {
	// Generate 5 of the same attestations.
	atts := make([]*ethpb.PendingAttestation, 5)
	for i := 0; i < len(atts); i++ {
		atts[i] = &ethpb.PendingAttestation{
			Data: &ethpb.AttestationData{Source: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
				Target: &ethpb.Checkpoint{Epoch: 0}},
			AggregationBits: bitfield.Bitlist{0x00, 0xFF, 0xFF, 0xFF},
		}
	}

	// Generate validators and state for the 5 attestations.
	validatorCount := 1000
	validators := make([]*ethpb.Validator, validatorCount)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}
	base := &ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	}
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)

	indices, err := epoch.UnslashedAttestingIndices(context.Background(), beaconState, atts)
	require.NoError(t, err)

	for i := 0; i < len(indices)-1; i++ {
		if indices[i] >= indices[i+1] {
			t.Error("sorted indices not sorted or duplicated")
		}
	}
}

func TestAttestingBalance_CorrectBalance(t *testing.T) {
	helpers.ClearCache()
	// Generate 2 attestations.
	atts := make([]*ethpb.PendingAttestation, 2)
	for i := 0; i < len(atts); i++ {
		atts[i] = &ethpb.PendingAttestation{
			Data: &ethpb.AttestationData{
				Target: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
				Source: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
				Slot:   primitives.Slot(i),
			},
			AggregationBits: bitfield.Bitlist{0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
				0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01},
		}
	}

	// Generate validators with balances and state for the 2 attestations.
	validators := make([]*ethpb.Validator, params.BeaconConfig().MinGenesisActiveValidatorCount)
	balances := make([]uint64, params.BeaconConfig().MinGenesisActiveValidatorCount)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance,
		}
		balances[i] = params.BeaconConfig().MaxEffectiveBalance
	}
	base := &ethpb.BeaconState{
		Slot:        2,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),

		Validators: validators,
		Balances:   balances,
	}
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)

	balance, err := epoch.AttestingBalance(context.Background(), beaconState, atts)
	require.NoError(t, err)
	wanted := 256 * params.BeaconConfig().MaxEffectiveBalance
	assert.Equal(t, wanted, balance)
}

func TestProcessSlashings_NotSlashed(t *testing.T) {
	base := &ethpb.BeaconState{
		Slot:       0,
		Validators: []*ethpb.Validator{{Slashed: true}},
		Balances:   []uint64{params.BeaconConfig().MaxEffectiveBalance},
		Slashings:  []uint64{0, 1e9},
	}
	s, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	newState, err := epoch.ProcessSlashings(s, params.BeaconConfig().ProportionalSlashingMultiplier)
	require.NoError(t, err)
	wanted := params.BeaconConfig().MaxEffectiveBalance
	assert.Equal(t, wanted, newState.Balances()[0], "Unexpected slashed balance")
}

func TestProcessSlashings_SlashedLess(t *testing.T) {
	tests := []struct {
		state *ethpb.BeaconState
		want  uint64
	}{
		{
			state: &ethpb.BeaconState{
				Validators: []*ethpb.Validator{
					{Slashed: true,
						WithdrawableEpoch: params.BeaconConfig().EpochsPerSlashingsVector / 2,
						EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance},
					{ExitEpoch: params.BeaconConfig().FarFutureEpoch, EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance}},
				Balances:  []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
				Slashings: []uint64{0, 1e9},
			},
			// penalty    = validator balance / increment * (2*total_penalties) / total_balance * increment
			// 1000000000 = (32 * 1e9)        / (1 * 1e9) * (1*1e9)             / (32*1e9)      * (1 * 1e9)
			want: uint64(31000000000), // 32 * 1e9 - 1000000000
		},
		{
			state: &ethpb.BeaconState{
				Validators: []*ethpb.Validator{
					{Slashed: true,
						WithdrawableEpoch: params.BeaconConfig().EpochsPerSlashingsVector / 2,
						EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance},
					{ExitEpoch: params.BeaconConfig().FarFutureEpoch, EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance},
					{ExitEpoch: params.BeaconConfig().FarFutureEpoch, EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance},
				},
				Balances:  []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
				Slashings: []uint64{0, 1e9},
			},
			// penalty    = validator balance / increment * (2*total_penalties) / total_balance * increment
			// 500000000 = (32 * 1e9)        / (1 * 1e9) * (1*1e9)             / (32*1e9)      * (1 * 1e9)
			want: uint64(32000000000), // 32 * 1e9 - 500000000
		},
		{
			state: &ethpb.BeaconState{
				Validators: []*ethpb.Validator{
					{Slashed: true,
						WithdrawableEpoch: params.BeaconConfig().EpochsPerSlashingsVector / 2,
						EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance},
					{ExitEpoch: params.BeaconConfig().FarFutureEpoch, EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance},
					{ExitEpoch: params.BeaconConfig().FarFutureEpoch, EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance},
				},
				Balances:  []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
				Slashings: []uint64{0, 2 * 1e9},
			},
			// penalty    = validator balance / increment * (3*total_penalties) / total_balance * increment
			// 1000000000 = (32 * 1e9)        / (1 * 1e9) * (1*2e9)             / (64*1e9)      * (1 * 1e9)
			want: uint64(31000000000), // 32 * 1e9 - 1000000000
		},
		{
			state: &ethpb.BeaconState{
				Validators: []*ethpb.Validator{
					{Slashed: true,
						WithdrawableEpoch: params.BeaconConfig().EpochsPerSlashingsVector / 2,
						EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance - params.BeaconConfig().EffectiveBalanceIncrement},
					{ExitEpoch: params.BeaconConfig().FarFutureEpoch, EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance - params.BeaconConfig().EffectiveBalanceIncrement}},
				Balances:  []uint64{params.BeaconConfig().MaxEffectiveBalance - params.BeaconConfig().EffectiveBalanceIncrement, params.BeaconConfig().MaxEffectiveBalance - params.BeaconConfig().EffectiveBalanceIncrement},
				Slashings: []uint64{0, 1e9},
			},
			// penalty    = validator balance           / increment * (3*total_penalties) / total_balance        * increment
			// 2000000000 = (32  * 1e9 - 1*1e9)         / (1 * 1e9) * (2*1e9)             / (31*1e9)             * (1 * 1e9)
			want: uint64(30000000000), // 32 * 1e9 - 2000000000
		},
	}

	for i, tt := range tests {
		t.Run(fmt.Sprint(i), func(t *testing.T) {
			original := proto.Clone(tt.state)
			s, err := state_native.InitializeFromProtoPhase0(tt.state)
			require.NoError(t, err)
			helpers.ClearCache()
			newState, err := epoch.ProcessSlashings(s, params.BeaconConfig().ProportionalSlashingMultiplier)
			require.NoError(t, err)
			assert.Equal(t, tt.want, newState.Balances()[0], "ProcessSlashings({%v}) = newState; newState.Balances[0] = %d", original, newState.Balances()[0])
		})
	}
}

func TestProcessFinalUpdates_CanProcess(t *testing.T) {
	s := buildState(t, params.BeaconConfig().SlotsPerHistoricalRoot-1, uint64(params.BeaconConfig().SlotsPerEpoch))
	ce := time.CurrentEpoch(s)
	ne := ce + 1
	require.NoError(t, s.SetEth1DataVotes([]*ethpb.Eth1Data{}))
	balances := s.Balances()
	balances[0] = 31.75 * 1e9
	balances[1] = 31.74 * 1e9
	require.NoError(t, s.SetBalances(balances))

	slashings := s.Slashings()
	slashings[ce] = 0
	require.NoError(t, s.SetSlashings(slashings))
	mixes := s.RandaoMixes()
	mixes[ce] = []byte{'A'}
	require.NoError(t, s.SetRandaoMixes(mixes))
	newS, err := epoch.ProcessFinalUpdates(s)
	require.NoError(t, err)

	// Verify effective balance is correctly updated.
	assert.Equal(t, params.BeaconConfig().MaxEffectiveBalance, newS.Validators()[0].EffectiveBalance, "Effective balance incorrectly updated")
	assert.Equal(t, uint64(31*1e9), newS.Validators()[1].EffectiveBalance, "Effective balance incorrectly updated")

	// Verify slashed balances correctly updated.
	assert.Equal(t, newS.Slashings()[ce], newS.Slashings()[ne], "Unexpected slashed balance")

	// Verify randao is correctly updated in the right position.
	mix, err := newS.RandaoMixAtIndex(uint64(ne))
	assert.NoError(t, err)
	assert.DeepNotEqual(t, params.BeaconConfig().ZeroHash[:], mix, "latest RANDAO still zero hashes")

	// Verify historical root accumulator was appended.
	roots, err := newS.HistoricalRoots()
	require.NoError(t, err)
	assert.Equal(t, 1, len(roots), "Unexpected slashed balance")
	currAtt, err := newS.CurrentEpochAttestations()
	require.NoError(t, err)
	assert.NotNil(t, currAtt, "Nil value stored in current epoch attestations instead of empty slice")
}

func TestProcessRegistryUpdates_NoRotation(t *testing.T) {
	base := &ethpb.BeaconState{
		Slot: 5 * params.BeaconConfig().SlotsPerEpoch,
		Validators: []*ethpb.Validator{
			{ExitEpoch: params.BeaconConfig().MaxSeedLookahead},
			{ExitEpoch: params.BeaconConfig().MaxSeedLookahead},
		},
		Balances: []uint64{
			params.BeaconConfig().MaxEffectiveBalance,
			params.BeaconConfig().MaxEffectiveBalance,
		},
		FinalizedCheckpoint: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
	}
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	newState, err := epoch.ProcessRegistryUpdates(context.Background(), beaconState)
	require.NoError(t, err)
	for i, validator := range newState.Validators() {
		assert.Equal(t, params.BeaconConfig().MaxSeedLookahead, validator.ExitEpoch, "Could not update registry %d", i)
	}
}

func TestProcessRegistryUpdates_EligibleToActivate(t *testing.T) {
	base := &ethpb.BeaconState{
		Slot:                5 * params.BeaconConfig().SlotsPerEpoch,
		FinalizedCheckpoint: &ethpb.Checkpoint{Epoch: 6, Root: make([]byte, fieldparams.RootLength)},
	}
	limit := helpers.ValidatorActivationChurnLimit(0)
	for i := uint64(0); i < limit+10; i++ {
		base.Validators = append(base.Validators, &ethpb.Validator{
			ActivationEligibilityEpoch: params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance:           params.BeaconConfig().MaxEffectiveBalance,
			ActivationEpoch:            params.BeaconConfig().FarFutureEpoch,
		})
	}
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	currentEpoch := time.CurrentEpoch(beaconState)
	newState, err := epoch.ProcessRegistryUpdates(context.Background(), beaconState)
	require.NoError(t, err)
	for i, validator := range newState.Validators() {
		assert.Equal(t, currentEpoch+1, validator.ActivationEligibilityEpoch, "Could not update registry %d, unexpected activation eligibility epoch", i)
		if uint64(i) < limit && validator.ActivationEpoch != helpers.ActivationExitEpoch(currentEpoch) {
			t.Errorf("Could not update registry %d, validators failed to activate: wanted activation epoch %d, got %d",
				i, helpers.ActivationExitEpoch(currentEpoch), validator.ActivationEpoch)
		}
		if uint64(i) >= limit && validator.ActivationEpoch != params.BeaconConfig().FarFutureEpoch {
			t.Errorf("Could not update registry %d, validators should not have been activated, wanted activation epoch: %d, got %d",
				i, params.BeaconConfig().FarFutureEpoch, validator.ActivationEpoch)
		}
	}
}

func TestProcessRegistryUpdates_EligibleToActivate_Cancun(t *testing.T) {
	base := &ethpb.BeaconStateDeneb{
		Slot:                5 * params.BeaconConfig().SlotsPerEpoch,
		FinalizedCheckpoint: &ethpb.Checkpoint{Epoch: 6, Root: make([]byte, fieldparams.RootLength)},
	}
	cfg := params.BeaconConfig()
	cfg.MinPerEpochChurnLimit = 10
	cfg.ChurnLimitQuotient = 1
	params.OverrideBeaconConfig(cfg)

	for i := uint64(0); i < 10; i++ {
		base.Validators = append(base.Validators, &ethpb.Validator{
			ActivationEligibilityEpoch: params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance:           params.BeaconConfig().MaxEffectiveBalance,
			ActivationEpoch:            params.BeaconConfig().FarFutureEpoch,
		})
	}
	beaconState, err := state_native.InitializeFromProtoDeneb(base)
	require.NoError(t, err)
	currentEpoch := time.CurrentEpoch(beaconState)
	newState, err := epoch.ProcessRegistryUpdates(context.Background(), beaconState)
	require.NoError(t, err)
	for i, validator := range newState.Validators() {
		assert.Equal(t, currentEpoch+1, validator.ActivationEligibilityEpoch, "Could not update registry %d, unexpected activation eligibility epoch", i)
		// Note: In Deneb, only validators indices before `MaxPerEpochActivationChurnLimit` should be activated.
		if uint64(i) < params.BeaconConfig().MaxPerEpochActivationChurnLimit && validator.ActivationEpoch != helpers.ActivationExitEpoch(currentEpoch) {
			t.Errorf("Could not update registry %d, validators failed to activate: wanted activation epoch %d, got %d",
				i, helpers.ActivationExitEpoch(currentEpoch), validator.ActivationEpoch)
		}
		if uint64(i) >= params.BeaconConfig().MaxPerEpochActivationChurnLimit && validator.ActivationEpoch != params.BeaconConfig().FarFutureEpoch {
			t.Errorf("Could not update registry %d, validators should not have been activated, wanted activation epoch: %d, got %d",
				i, params.BeaconConfig().FarFutureEpoch, validator.ActivationEpoch)
		}
	}
}

func TestProcessRegistryUpdates_ActivationCompletes(t *testing.T) {
	base := &ethpb.BeaconState{
		Slot: 5 * params.BeaconConfig().SlotsPerEpoch,
		Validators: []*ethpb.Validator{
			{ExitEpoch: params.BeaconConfig().MaxSeedLookahead,
				ActivationEpoch: 5 + params.BeaconConfig().MaxSeedLookahead + 1},
			{ExitEpoch: params.BeaconConfig().MaxSeedLookahead,
				ActivationEpoch: 5 + params.BeaconConfig().MaxSeedLookahead + 1},
		},
		FinalizedCheckpoint: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
	}
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	newState, err := epoch.ProcessRegistryUpdates(context.Background(), beaconState)
	require.NoError(t, err)
	for i, validator := range newState.Validators() {
		assert.Equal(t, params.BeaconConfig().MaxSeedLookahead, validator.ExitEpoch, "Could not update registry %d, unexpected exit slot", i)
	}
}

func TestProcessRegistryUpdates_ValidatorsEjected(t *testing.T) {
	base := &ethpb.BeaconState{
		Slot: 0,
		Validators: []*ethpb.Validator{
			{
				ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
				EffectiveBalance: params.BeaconConfig().EjectionBalance - 1,
			},
			{
				ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
				EffectiveBalance: params.BeaconConfig().EjectionBalance - 1,
			},
		},
		FinalizedCheckpoint: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
	}
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	newState, err := epoch.ProcessRegistryUpdates(context.Background(), beaconState)
	require.NoError(t, err)
	for i, validator := range newState.Validators() {
		assert.Equal(t, params.BeaconConfig().MaxSeedLookahead+1, validator.ExitEpoch, "Could not update registry %d, unexpected exit slot", i)
	}
}

func TestProcessRegistryUpdates_CanExits(t *testing.T) {
	e := primitives.Epoch(5)
	exitEpoch := helpers.ActivationExitEpoch(e)
	minWithdrawalDelay := params.BeaconConfig().MinValidatorWithdrawabilityDelay
	base := &ethpb.BeaconState{
		Slot: params.BeaconConfig().SlotsPerEpoch.Mul(uint64(e)),
		Validators: []*ethpb.Validator{
			{
				ExitEpoch:         exitEpoch,
				WithdrawableEpoch: exitEpoch + minWithdrawalDelay},
			{
				ExitEpoch:         exitEpoch,
				WithdrawableEpoch: exitEpoch + minWithdrawalDelay},
		},
		FinalizedCheckpoint: &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
	}
	beaconState, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	newState, err := epoch.ProcessRegistryUpdates(context.Background(), beaconState)
	require.NoError(t, err)
	for i, validator := range newState.Validators() {
		assert.Equal(t, exitEpoch, validator.ExitEpoch, "Could not update registry %d, unexpected exit slot", i)
	}
}

func buildState(t testing.TB, slot primitives.Slot, validatorCount uint64) state.BeaconState {
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
	s, err := util.NewBeaconState()
	require.NoError(t, err)
	if err := s.SetSlot(slot); err != nil {
		t.Error(err)
	}
	if err := s.SetBalances(validatorBalances); err != nil {
		t.Error(err)
	}
	if err := s.SetValidators(validators); err != nil {
		t.Error(err)
	}
	return s
}

func TestProcessSlashings_BadValue(t *testing.T) {
	base := &ethpb.BeaconState{
		Slot:       0,
		Validators: []*ethpb.Validator{{Slashed: true}},
		Balances:   []uint64{params.BeaconConfig().MaxEffectiveBalance},
		Slashings:  []uint64{math.MaxUint64, 1e9},
	}
	s, err := state_native.InitializeFromProtoPhase0(base)
	require.NoError(t, err)
	_, err = epoch.ProcessSlashings(s, params.BeaconConfig().ProportionalSlashingMultiplier)
	require.ErrorContains(t, "addition overflows", err)
}

func TestProcessHistoricalDataUpdate(t *testing.T) {
	tests := []struct {
		name     string
		st       func() state.BeaconState
		verifier func(state.BeaconState)
	}{
		{
			name: "no change",
			st: func() state.BeaconState {
				st, _ := util.DeterministicGenesisState(t, 1)
				return st
			},
			verifier: func(st state.BeaconState) {
				roots, err := st.HistoricalRoots()
				require.NoError(t, err)
				require.Equal(t, 0, len(roots))
			},
		},
		{
			name: "before capella can process and get historical root",
			st: func() state.BeaconState {
				st, _ := util.DeterministicGenesisState(t, 1)
				st, err := transition.ProcessSlots(context.Background(), st, params.BeaconConfig().SlotsPerHistoricalRoot-1)
				require.NoError(t, err)
				return st
			},
			verifier: func(st state.BeaconState) {
				roots, err := st.HistoricalRoots()
				require.NoError(t, err)
				require.Equal(t, 1, len(roots))

				b := &ethpb.HistoricalBatch{
					BlockRoots: st.BlockRoots(),
					StateRoots: st.StateRoots(),
				}
				r, err := b.HashTreeRoot()
				require.NoError(t, err)
				require.DeepEqual(t, r[:], roots[0])

				_, err = st.HistoricalSummaries()
				require.ErrorContains(t, "HistoricalSummaries is not supported for phase0", err)
			},
		},
		{
			name: "after capella can process and get historical summary",
			st: func() state.BeaconState {
				st, _ := util.DeterministicGenesisStateCapella(t, 1)
				st, err := transition.ProcessSlots(context.Background(), st, params.BeaconConfig().SlotsPerHistoricalRoot-1)
				require.NoError(t, err)
				return st
			},
			verifier: func(st state.BeaconState) {
				summaries, err := st.HistoricalSummaries()
				require.NoError(t, err)
				require.Equal(t, 1, len(summaries))

				br, err := stateutil.ArraysRoot(st.BlockRoots(), fieldparams.BlockRootsLength)
				require.NoError(t, err)
				sr, err := stateutil.ArraysRoot(st.StateRoots(), fieldparams.StateRootsLength)
				require.NoError(t, err)
				b := &ethpb.HistoricalSummary{
					BlockSummaryRoot: br[:],
					StateSummaryRoot: sr[:],
				}
				require.DeepEqual(t, b, summaries[0])
				hrs, err := st.HistoricalRoots()
				require.NoError(t, err)
				require.DeepEqual(t, hrs, [][]byte{})
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := epoch.ProcessHistoricalDataUpdate(tt.st())
			require.NoError(t, err)
			tt.verifier(got)
		})
	}
}
