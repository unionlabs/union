package helpers

import (
	"context"
	"fmt"
	"strconv"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestComputeCommittee_WithoutCache(t *testing.T) {
	ClearCache()

	// Create 10 committees
	committeeCount := uint64(10)
	validatorCount := committeeCount * params.BeaconConfig().TargetCommitteeSize
	validators := make([]*ethpb.Validator, validatorCount)

	for i := 0; i < len(validators); i++ {
		k := make([]byte, 48)
		copy(k, strconv.Itoa(i))
		validators[i] = &ethpb.Validator{
			PublicKey:             k,
			WithdrawalCredentials: make([]byte, 32),
			ExitEpoch:             params.BeaconConfig().FarFutureEpoch,
		}
	}

	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		Slot:        200,
		BlockRoots:  make([][]byte, params.BeaconConfig().SlotsPerHistoricalRoot),
		StateRoots:  make([][]byte, params.BeaconConfig().SlotsPerHistoricalRoot),
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)

	epoch := time.CurrentEpoch(state)
	indices, err := ActiveValidatorIndices(context.Background(), state, epoch)
	require.NoError(t, err)
	seed, err := Seed(state, epoch, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)
	committees, err := computeCommittee(indices, seed, 0, 1 /* Total committee*/)
	assert.NoError(t, err, "Could not compute committee")

	// Test shuffled indices are correct for index 5 committee
	index := uint64(5)
	committee5, err := computeCommittee(indices, seed, index, committeeCount)
	assert.NoError(t, err, "Could not compute committee")
	start := slice.SplitOffset(validatorCount, committeeCount, index)
	end := slice.SplitOffset(validatorCount, committeeCount, index+1)
	assert.DeepEqual(t, committee5, committees[start:end], "Committee has different shuffled indices")

	// Test shuffled indices are correct for index 9 committee
	index = uint64(9)
	committee9, err := computeCommittee(indices, seed, index, committeeCount)
	assert.NoError(t, err, "Could not compute committee")
	start = slice.SplitOffset(validatorCount, committeeCount, index)
	end = slice.SplitOffset(validatorCount, committeeCount, index+1)
	assert.DeepEqual(t, committee9, committees[start:end], "Committee has different shuffled indices")
}

func TestComputeCommittee_RegressionTest(t *testing.T) {
	ClearCache()

	indices := []primitives.ValidatorIndex{1, 3, 8, 16, 18, 19, 20, 23, 30, 35, 43, 46, 47, 54, 56, 58, 69, 70, 71, 83, 84, 85, 91, 96, 100, 103, 105, 106, 112, 121, 127, 128, 129, 140, 142, 144, 146, 147, 149, 152, 153, 154, 157, 160, 173, 175, 180, 182, 188, 189, 191, 194, 201, 204, 217, 221, 226, 228, 230, 231, 239, 241, 249, 250, 255}
	seed := [32]byte{68, 110, 161, 250, 98, 230, 161, 172, 227, 226, 99, 11, 138, 124, 201, 134, 38, 197, 0, 120, 6, 165, 122, 34, 19, 216, 43, 226, 210, 114, 165, 183}
	index := uint64(215)
	count := uint64(32)
	_, err := computeCommittee(indices, seed, index, count)
	require.ErrorContains(t, "index out of range", err)
}

func TestVerifyBitfieldLength_OK(t *testing.T) {
	ClearCache()

	bf := bitfield.Bitlist{0xFF, 0x01}
	committeeSize := uint64(8)
	assert.NoError(t, VerifyBitfieldLength(bf, committeeSize), "Bitfield is not validated when it was supposed to be")

	bf = bitfield.Bitlist{0xFF, 0x07}
	committeeSize = 10
	assert.NoError(t, VerifyBitfieldLength(bf, committeeSize), "Bitfield is not validated when it was supposed to be")
}

func TestCommitteeAssignments_CannotRetrieveFutureEpoch(t *testing.T) {
	ClearCache()

	epoch := primitives.Epoch(1)
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Slot: 0, // Epoch 0.
	})
	require.NoError(t, err)
	_, _, err = CommitteeAssignments(context.Background(), state, epoch+1)
	assert.ErrorContains(t, "can't be greater than next epoch", err)
}

func TestCommitteeAssignments_NoProposerForSlot0(t *testing.T) {
	ClearCache()

	validators := make([]*ethpb.Validator, 4*params.BeaconConfig().SlotsPerEpoch)
	for i := 0; i < len(validators); i++ {
		var activationEpoch primitives.Epoch
		if i >= len(validators)/2 {
			activationEpoch = 3
		}
		validators[i] = &ethpb.Validator{
			ActivationEpoch: activationEpoch,
			ExitEpoch:       params.BeaconConfig().FarFutureEpoch,
		}
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		Slot:        2 * params.BeaconConfig().SlotsPerEpoch, // epoch 2
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)
	_, proposerIndexToSlots, err := CommitteeAssignments(context.Background(), state, 0)
	require.NoError(t, err, "Failed to determine CommitteeAssignments")
	for _, ss := range proposerIndexToSlots {
		for _, s := range ss {
			assert.NotEqual(t, uint64(0), s, "No proposer should be assigned to slot 0")
		}
	}
}

func TestCommitteeAssignments_CanRetrieve(t *testing.T) {
	// Initialize test with 256 validators, each slot and each index gets 4 validators.
	validators := make([]*ethpb.Validator, 4*params.BeaconConfig().SlotsPerEpoch)
	for i := 0; i < len(validators); i++ {
		// First 2 epochs only half validators are activated.
		var activationEpoch primitives.Epoch
		if i >= len(validators)/2 {
			activationEpoch = 3
		}
		validators[i] = &ethpb.Validator{
			ActivationEpoch: activationEpoch,
			ExitEpoch:       params.BeaconConfig().FarFutureEpoch,
		}
	}

	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		Slot:        2 * params.BeaconConfig().SlotsPerEpoch, // epoch 2
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)

	tests := []struct {
		index          primitives.ValidatorIndex
		slot           primitives.Slot
		committee      []primitives.ValidatorIndex
		committeeIndex primitives.CommitteeIndex
		isProposer     bool
		proposerSlot   primitives.Slot
	}{
		{
			index:          0,
			slot:           78,
			committee:      []primitives.ValidatorIndex{0, 38},
			committeeIndex: 0,
			isProposer:     false,
		},
		{
			index:          1,
			slot:           71,
			committee:      []primitives.ValidatorIndex{1, 4},
			committeeIndex: 0,
			isProposer:     true,
			proposerSlot:   79,
		},
		{
			index:          11,
			slot:           90,
			committee:      []primitives.ValidatorIndex{31, 11},
			committeeIndex: 0,
			isProposer:     false,
		}, {
			index:          2,
			slot:           127, // 3rd epoch has more active validators
			committee:      []primitives.ValidatorIndex{89, 2, 81, 5},
			committeeIndex: 0,
			isProposer:     false,
		},
	}

	for i, tt := range tests {
		t.Run(fmt.Sprintf("%d", i), func(t *testing.T) {
			ClearCache()

			validatorIndexToCommittee, proposerIndexToSlots, err := CommitteeAssignments(context.Background(), state, slots.ToEpoch(tt.slot))
			require.NoError(t, err, "Failed to determine CommitteeAssignments")
			cac := validatorIndexToCommittee[tt.index]
			assert.Equal(t, tt.committeeIndex, cac.CommitteeIndex, "Unexpected committeeIndex for validator index %d", tt.index)
			assert.Equal(t, tt.slot, cac.AttesterSlot, "Unexpected slot for validator index %d", tt.index)
			if len(proposerIndexToSlots[tt.index]) > 0 && proposerIndexToSlots[tt.index][0] != tt.proposerSlot {
				t.Errorf("wanted proposer slot %d, got proposer slot %d for validator index %d",
					tt.proposerSlot, proposerIndexToSlots[tt.index][0], tt.index)
			}
			assert.DeepEqual(t, tt.committee, cac.Committee, "Unexpected committee for validator index %d", tt.index)
		})
	}
}

func TestCommitteeAssignments_CannotRetrieveFuture(t *testing.T) {
	ClearCache()

	// Initialize test with 256 validators, each slot and each index gets 4 validators.
	validators := make([]*ethpb.Validator, 4*params.BeaconConfig().SlotsPerEpoch)
	for i := 0; i < len(validators); i++ {
		// First 2 epochs only half validators are activated.
		var activationEpoch primitives.Epoch
		if i >= len(validators)/2 {
			activationEpoch = 3
		}
		validators[i] = &ethpb.Validator{
			ActivationEpoch: activationEpoch,
			ExitEpoch:       params.BeaconConfig().FarFutureEpoch,
		}
	}

	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		Slot:        2 * params.BeaconConfig().SlotsPerEpoch, // epoch 2
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)
	_, proposerIndxs, err := CommitteeAssignments(context.Background(), state, time.CurrentEpoch(state))
	require.NoError(t, err)
	require.NotEqual(t, 0, len(proposerIndxs), "wanted non-zero proposer index set")

	_, proposerIndxs, err = CommitteeAssignments(context.Background(), state, time.CurrentEpoch(state)+1)
	require.NoError(t, err)
	require.NotEqual(t, 0, len(proposerIndxs), "wanted non-zero proposer index set")
}

func TestCommitteeAssignments_CannotRetrieveOlderThanSlotsPerHistoricalRoot(t *testing.T) {
	ClearCache()

	// Initialize test with 256 validators, each slot and each index gets 4 validators.
	validators := make([]*ethpb.Validator, 4*params.BeaconConfig().SlotsPerEpoch)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}

	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		Slot:        params.BeaconConfig().SlotsPerHistoricalRoot + 1,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)
	_, _, err = CommitteeAssignments(context.Background(), state, 0)
	require.ErrorContains(t, "start slot 0 is smaller than the minimum valid start slot 1", err)
}

func TestCommitteeAssignments_EverySlotHasMin1Proposer(t *testing.T) {
	ClearCache()

	// Initialize test with 256 validators, each slot and each index gets 4 validators.
	validators := make([]*ethpb.Validator, 4*params.BeaconConfig().SlotsPerEpoch)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ActivationEpoch: 0,
			ExitEpoch:       params.BeaconConfig().FarFutureEpoch,
		}
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		Slot:        2 * params.BeaconConfig().SlotsPerEpoch, // epoch 2
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)
	epoch := primitives.Epoch(1)
	_, proposerIndexToSlots, err := CommitteeAssignments(context.Background(), state, epoch)
	require.NoError(t, err, "Failed to determine CommitteeAssignments")

	slotsWithProposers := make(map[primitives.Slot]bool)
	for _, proposerSlots := range proposerIndexToSlots {
		for _, slot := range proposerSlots {
			slotsWithProposers[slot] = true
		}
	}
	assert.Equal(t, uint64(params.BeaconConfig().SlotsPerEpoch), uint64(len(slotsWithProposers)), "Unexpected slots")
	startSlot, err := slots.EpochStart(epoch)
	require.NoError(t, err)
	endSlot, err := slots.EpochStart(epoch + 1)
	require.NoError(t, err)
	for i := startSlot; i < endSlot; i++ {
		hasProposer := slotsWithProposers[i]
		assert.Equal(t, true, hasProposer, "Expected every slot in epoch 1 to have a proposer, slot %d did not", i)
	}
}

func TestVerifyAttestationBitfieldLengths_OK(t *testing.T) {
	validators := make([]*ethpb.Validator, 2*params.BeaconConfig().SlotsPerEpoch)
	activeRoots := make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}

	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: activeRoots,
	})
	require.NoError(t, err)

	tests := []struct {
		attestation         *ethpb.Attestation
		stateSlot           primitives.Slot
		verificationFailure bool
	}{
		{
			attestation: &ethpb.Attestation{
				AggregationBits: bitfield.Bitlist{0x05},
				Data: &ethpb.AttestationData{
					CommitteeIndex: 5,
					Target:         &ethpb.Checkpoint{Root: make([]byte, 32)},
				},
			},
			stateSlot: 5,
		},
		{

			attestation: &ethpb.Attestation{
				AggregationBits: bitfield.Bitlist{0x06},
				Data: &ethpb.AttestationData{
					CommitteeIndex: 10,
					Target:         &ethpb.Checkpoint{Root: make([]byte, 32)},
				},
			},
			stateSlot: 10,
		},
		{
			attestation: &ethpb.Attestation{
				AggregationBits: bitfield.Bitlist{0x06},
				Data: &ethpb.AttestationData{
					CommitteeIndex: 20,
					Target:         &ethpb.Checkpoint{Root: make([]byte, 32)},
				},
			},
			stateSlot: 20,
		},
		{
			attestation: &ethpb.Attestation{
				AggregationBits: bitfield.Bitlist{0x06},
				Data: &ethpb.AttestationData{
					CommitteeIndex: 20,
					Target:         &ethpb.Checkpoint{Root: make([]byte, 32)},
				},
			},
			stateSlot: 20,
		},
		{
			attestation: &ethpb.Attestation{
				AggregationBits: bitfield.Bitlist{0xFF, 0xC0, 0x01},
				Data: &ethpb.AttestationData{
					CommitteeIndex: 5,
					Target:         &ethpb.Checkpoint{Root: make([]byte, 32)},
				},
			},
			stateSlot:           5,
			verificationFailure: true,
		},
		{
			attestation: &ethpb.Attestation{
				AggregationBits: bitfield.Bitlist{0xFF, 0x01},
				Data: &ethpb.AttestationData{
					CommitteeIndex: 20,
					Target:         &ethpb.Checkpoint{Root: make([]byte, 32)},
				},
			},
			stateSlot:           20,
			verificationFailure: true,
		},
	}

	for i, tt := range tests {
		ClearCache()

		require.NoError(t, state.SetSlot(tt.stateSlot))
		err := VerifyAttestationBitfieldLengths(context.Background(), state, tt.attestation)
		if tt.verificationFailure {
			assert.NotNil(t, err, "Verification succeeded when it was supposed to fail")
		} else {
			assert.NoError(t, err, "%d Failed to verify bitfield: %v", i, err)
		}
	}
}

func TestUpdateCommitteeCache_CanUpdate(t *testing.T) {
	ClearCache()

	validatorCount := params.BeaconConfig().MinGenesisActiveValidatorCount
	validators := make([]*ethpb.Validator, validatorCount)
	indices := make([]primitives.ValidatorIndex, validatorCount)
	for i := primitives.ValidatorIndex(0); uint64(i) < validatorCount; i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance: 1,
		}
		indices[i] = i
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)
	require.NoError(t, UpdateCommitteeCache(context.Background(), state, time.CurrentEpoch(state)))

	epoch := primitives.Epoch(0)
	idx := primitives.CommitteeIndex(1)
	seed, err := Seed(state, epoch, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)

	indices, err = committeeCache.Committee(context.Background(), params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch)), seed, idx)
	require.NoError(t, err)
	assert.Equal(t, params.BeaconConfig().TargetCommitteeSize, uint64(len(indices)), "Did not save correct indices lengths")
}

func TestUpdateCommitteeCache_CanUpdateAcrossEpochs(t *testing.T) {
	ClearCache()

	validatorCount := params.BeaconConfig().MinGenesisActiveValidatorCount
	validators := make([]*ethpb.Validator, validatorCount)
	indices := make([]primitives.ValidatorIndex, validatorCount)
	for i := primitives.ValidatorIndex(0); uint64(i) < validatorCount; i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance: 1,
		}
		indices[i] = i
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)
	e := time.CurrentEpoch(state)
	require.NoError(t, UpdateCommitteeCache(context.Background(), state, e))

	seed, err := Seed(state, e, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)
	require.Equal(t, true, committeeCache.HasEntry(string(seed[:])))

	nextSeed, err := Seed(state, e+1, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)
	require.Equal(t, false, committeeCache.HasEntry(string(nextSeed[:])))

	require.NoError(t, UpdateCommitteeCache(context.Background(), state, e+1))

	require.Equal(t, true, committeeCache.HasEntry(string(nextSeed[:])))
}

func BenchmarkComputeCommittee300000_WithPreCache(b *testing.B) {
	validators := make([]*ethpb.Validator, 300000)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(b, err)

	epoch := time.CurrentEpoch(state)
	indices, err := ActiveValidatorIndices(context.Background(), state, epoch)
	require.NoError(b, err)
	seed, err := Seed(state, epoch, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(b, err)

	index := uint64(3)
	_, err = computeCommittee(indices, seed, index, params.BeaconConfig().MaxCommitteesPerSlot)
	if err != nil {
		panic(err)
	}

	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		_, err := computeCommittee(indices, seed, index, params.BeaconConfig().MaxCommitteesPerSlot)
		if err != nil {
			panic(err)
		}
	}
}

func BenchmarkComputeCommittee3000000_WithPreCache(b *testing.B) {
	validators := make([]*ethpb.Validator, 3000000)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(b, err)

	epoch := time.CurrentEpoch(state)
	indices, err := ActiveValidatorIndices(context.Background(), state, epoch)
	require.NoError(b, err)
	seed, err := Seed(state, epoch, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(b, err)

	index := uint64(3)
	_, err = computeCommittee(indices, seed, index, params.BeaconConfig().MaxCommitteesPerSlot)
	if err != nil {
		panic(err)
	}

	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		_, err := computeCommittee(indices, seed, index, params.BeaconConfig().MaxCommitteesPerSlot)
		if err != nil {
			panic(err)
		}
	}
}

func BenchmarkComputeCommittee128000_WithOutPreCache(b *testing.B) {
	validators := make([]*ethpb.Validator, 128000)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(b, err)

	epoch := time.CurrentEpoch(state)
	indices, err := ActiveValidatorIndices(context.Background(), state, epoch)
	require.NoError(b, err)
	seed, err := Seed(state, epoch, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(b, err)

	i := uint64(0)
	index := uint64(0)
	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		i++
		_, err := computeCommittee(indices, seed, index, params.BeaconConfig().MaxCommitteesPerSlot)
		if err != nil {
			panic(err)
		}
		if i < params.BeaconConfig().TargetCommitteeSize {
			index = (index + 1) % params.BeaconConfig().MaxCommitteesPerSlot
			i = 0
		}
	}
}

func BenchmarkComputeCommittee1000000_WithOutCache(b *testing.B) {
	validators := make([]*ethpb.Validator, 1000000)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(b, err)

	epoch := time.CurrentEpoch(state)
	indices, err := ActiveValidatorIndices(context.Background(), state, epoch)
	require.NoError(b, err)
	seed, err := Seed(state, epoch, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(b, err)

	i := uint64(0)
	index := uint64(0)
	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		i++
		_, err := computeCommittee(indices, seed, index, params.BeaconConfig().MaxCommitteesPerSlot)
		if err != nil {
			panic(err)
		}
		if i < params.BeaconConfig().TargetCommitteeSize {
			index = (index + 1) % params.BeaconConfig().MaxCommitteesPerSlot
			i = 0
		}
	}
}

func BenchmarkComputeCommittee4000000_WithOutCache(b *testing.B) {
	validators := make([]*ethpb.Validator, 4000000)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(b, err)

	epoch := time.CurrentEpoch(state)
	indices, err := ActiveValidatorIndices(context.Background(), state, epoch)
	require.NoError(b, err)
	seed, err := Seed(state, epoch, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(b, err)

	i := uint64(0)
	index := uint64(0)
	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		i++
		_, err := computeCommittee(indices, seed, index, params.BeaconConfig().MaxCommitteesPerSlot)
		if err != nil {
			panic(err)
		}
		if i < params.BeaconConfig().TargetCommitteeSize {
			index = (index + 1) % params.BeaconConfig().MaxCommitteesPerSlot
			i = 0
		}
	}
}

func TestBeaconCommitteeFromState_UpdateCacheForPreviousEpoch(t *testing.T) {
	committeeSize := uint64(16)
	validators := make([]*ethpb.Validator, params.BeaconConfig().SlotsPerEpoch.Mul(committeeSize))
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}

	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Slot:        params.BeaconConfig().SlotsPerEpoch,
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)
	_, err = BeaconCommitteeFromState(context.Background(), state, 1 /* previous epoch */, 0)
	require.NoError(t, err)

	// Verify previous epoch is cached
	seed, err := Seed(state, 0, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)
	activeIndices, err := committeeCache.ActiveIndices(context.Background(), seed)
	require.NoError(t, err)
	assert.NotNil(t, activeIndices, "Did not cache active indices")
}

func TestPrecomputeProposerIndices_Ok(t *testing.T) {
	validators := make([]*ethpb.Validator, params.BeaconConfig().MinGenesisActiveValidatorCount)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}

	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)

	indices, err := ActiveValidatorIndices(context.Background(), state, 0)
	require.NoError(t, err)

	proposerIndices, err := precomputeProposerIndices(state, indices, time.CurrentEpoch(state))
	require.NoError(t, err)

	var wantedProposerIndices []primitives.ValidatorIndex
	seed, err := Seed(state, 0, params.BeaconConfig().DomainBeaconProposer)
	require.NoError(t, err)
	for i := uint64(0); i < uint64(params.BeaconConfig().SlotsPerEpoch); i++ {
		seedWithSlot := append(seed[:], bytesutil.Bytes8(i)...)
		seedWithSlotHash := hash.Hash(seedWithSlot)
		index, err := ComputeProposerIndex(state, indices, seedWithSlotHash)
		require.NoError(t, err)
		wantedProposerIndices = append(wantedProposerIndices, index)
	}
	assert.DeepEqual(t, wantedProposerIndices, proposerIndices, "Did not precompute proposer indices correctly")
}
