package helpers_test

import (
	"context"
	"strconv"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestAttestation_IsAggregator(t *testing.T) {
	t.Run("aggregator", func(t *testing.T) {
		helpers.ClearCache()

		beaconState, privKeys := util.DeterministicGenesisState(t, 100)
		committee, err := helpers.BeaconCommitteeFromState(context.Background(), beaconState, 0, 0)
		require.NoError(t, err)
		sig := privKeys[0].Sign([]byte{'A'})
		agg, err := helpers.IsAggregator(uint64(len(committee)), sig.Marshal())
		require.NoError(t, err)
		assert.Equal(t, true, agg, "Wanted aggregator true")
	})

	t.Run("not aggregator", func(t *testing.T) {
		helpers.ClearCache()

		params.SetupTestConfigCleanup(t)
		params.OverrideBeaconConfig(params.MinimalSpecConfig())
		beaconState, privKeys := util.DeterministicGenesisState(t, 2048)

		committee, err := helpers.BeaconCommitteeFromState(context.Background(), beaconState, 0, 0)
		require.NoError(t, err)
		sig := privKeys[0].Sign([]byte{'A'})
		agg, err := helpers.IsAggregator(uint64(len(committee)), sig.Marshal())
		require.NoError(t, err)
		assert.Equal(t, false, agg, "Wanted aggregator false")
	})
}

func TestAttestation_ComputeSubnetForAttestation(t *testing.T) {
	helpers.ClearCache()

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
	att := &ethpb.Attestation{
		AggregationBits: []byte{'A'},
		Data: &ethpb.AttestationData{
			Slot:            34,
			CommitteeIndex:  4,
			BeaconBlockRoot: []byte{'C'},
			Source:          nil,
			Target:          nil,
		},
		Signature: []byte{'B'},
	}
	valCount, err := helpers.ActiveValidatorCount(context.Background(), state, slots.ToEpoch(att.Data.Slot))
	require.NoError(t, err)
	sub := helpers.ComputeSubnetForAttestation(valCount, att)
	assert.Equal(t, uint64(6), sub, "Did not get correct subnet for attestation")
}

func Test_ValidateAttestationTime(t *testing.T) {
	cfg := params.BeaconConfig().Copy()
	cfg.DenebForkEpoch = 5
	params.OverrideBeaconConfig(cfg)
	params.SetupTestConfigCleanup(t)

	if params.BeaconConfig().MaximumGossipClockDisparityDuration() < 200*time.Millisecond {
		t.Fatal("This test expects the maximum clock disparity to be at least 200ms")
	}

	type args struct {
		attSlot     primitives.Slot
		genesisTime time.Time
	}
	tests := []struct {
		name      string
		args      args
		wantedErr string
	}{
		{
			name: "attestation.slot == current_slot",
			args: args{
				attSlot:     15,
				genesisTime: prysmTime.Now().Add(-15 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second),
			},
		},
		{
			name: "attestation.slot == current_slot, received in middle of slot",
			args: args{
				attSlot: 15,
				genesisTime: prysmTime.Now().Add(
					-15 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second,
				).Add(-(time.Duration(params.BeaconConfig().SecondsPerSlot/2) * time.Second)),
			},
		},
		{
			name: "attestation.slot == current_slot, received 200ms early",
			args: args{
				attSlot: 16,
				genesisTime: prysmTime.Now().Add(
					-16 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second,
				).Add(-200 * time.Millisecond),
			},
		},
		{
			name: "attestation.slot > current_slot",
			args: args{
				attSlot:     16,
				genesisTime: prysmTime.Now().Add(-15 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second),
			},
			wantedErr: "not within attestation propagation range",
		},
		{
			name: "attestation.slot < current_slot-ATTESTATION_PROPAGATION_SLOT_RANGE",
			args: args{
				attSlot:     100 - params.BeaconConfig().AttestationPropagationSlotRange - 1,
				genesisTime: prysmTime.Now().Add(-100 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second),
			},
			wantedErr: "not within attestation propagation range",
		},
		{
			name: "attestation.slot = current_slot-ATTESTATION_PROPAGATION_SLOT_RANGE",
			args: args{
				attSlot:     100 - params.BeaconConfig().AttestationPropagationSlotRange,
				genesisTime: prysmTime.Now().Add(-100 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second),
			},
		},
		{
			name: "attestation.slot = current_slot-ATTESTATION_PROPAGATION_SLOT_RANGE, received 200ms late",
			args: args{
				attSlot: 100 - params.BeaconConfig().AttestationPropagationSlotRange,
				genesisTime: prysmTime.Now().Add(
					-100 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second,
				).Add(200 * time.Millisecond),
			},
		},
		{
			name: "attestation.slot < current_slot-ATTESTATION_PROPAGATION_SLOT_RANGE in deneb",
			args: args{
				attSlot:     300 - params.BeaconConfig().AttestationPropagationSlotRange - 1,
				genesisTime: prysmTime.Now().Add(-300 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second),
			},
		},
		{
			name: "attestation.slot = current_slot-ATTESTATION_PROPAGATION_SLOT_RANGE in deneb",
			args: args{
				attSlot:     300 - params.BeaconConfig().AttestationPropagationSlotRange,
				genesisTime: prysmTime.Now().Add(-300 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second),
			},
		},
		{
			name: "attestation.slot = current_slot-ATTESTATION_PROPAGATION_SLOT_RANGE, received 200ms late in deneb",
			args: args{
				attSlot: 300 - params.BeaconConfig().AttestationPropagationSlotRange,
				genesisTime: prysmTime.Now().Add(
					-300 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second,
				).Add(200 * time.Millisecond),
			},
		},
		{
			name: "attestation.slot != current epoch or previous epoch in deneb",
			args: args{
				attSlot: 300 - params.BeaconConfig().AttestationPropagationSlotRange,
				genesisTime: prysmTime.Now().Add(
					-500 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second,
				).Add(200 * time.Millisecond),
			},
			wantedErr: "attestation epoch 8 not within current epoch 15 or previous epoch 14",
		},
		{
			name: "attestation.slot is well beyond current slot",
			args: args{
				attSlot:     1 << 32,
				genesisTime: prysmTime.Now().Add(-15 * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second),
			},
			wantedErr: "which exceeds max allowed value relative to the local clock",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			helpers.ClearCache()

			err := helpers.ValidateAttestationTime(tt.args.attSlot, tt.args.genesisTime,
				params.BeaconConfig().MaximumGossipClockDisparityDuration())
			if tt.wantedErr != "" {
				assert.ErrorContains(t, tt.wantedErr, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestVerifyCheckpointEpoch_Ok(t *testing.T) {
	helpers.ClearCache()

	// Genesis was 6 epochs ago exactly.
	offset := params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot * 6)
	genesis := time.Now().Add(-1 * time.Second * time.Duration(offset))
	assert.Equal(t, true, helpers.VerifyCheckpointEpoch(&ethpb.Checkpoint{Epoch: 6}, genesis))
	assert.Equal(t, true, helpers.VerifyCheckpointEpoch(&ethpb.Checkpoint{Epoch: 5}, genesis))
	assert.Equal(t, false, helpers.VerifyCheckpointEpoch(&ethpb.Checkpoint{Epoch: 4}, genesis))
	assert.Equal(t, false, helpers.VerifyCheckpointEpoch(&ethpb.Checkpoint{Epoch: 2}, genesis))
}

func TestValidateNilAttestation(t *testing.T) {
	tests := []struct {
		name        string
		attestation *ethpb.Attestation
		errString   string
	}{
		{
			name:        "nil attestation",
			attestation: nil,
			errString:   "attestation can't be nil",
		},
		{
			name:        "nil attestation data",
			attestation: &ethpb.Attestation{},
			errString:   "attestation's data can't be nil",
		},
		{
			name: "nil attestation source",
			attestation: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					Source: nil,
					Target: &ethpb.Checkpoint{},
				},
			},
			errString: "attestation's source can't be nil",
		},
		{
			name: "nil attestation target",
			attestation: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					Target: nil,
					Source: &ethpb.Checkpoint{},
				},
			},
			errString: "attestation's target can't be nil",
		},
		{
			name: "nil attestation bitfield",
			attestation: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					Target: &ethpb.Checkpoint{},
					Source: &ethpb.Checkpoint{},
				},
			},
			errString: "attestation's bitfield can't be nil",
		},
		{
			name: "good attestation",
			attestation: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					Target: &ethpb.Checkpoint{},
					Source: &ethpb.Checkpoint{},
				},
				AggregationBits: []byte{},
			},
			errString: "",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			helpers.ClearCache()

			if tt.errString != "" {
				require.ErrorContains(t, tt.errString, helpers.ValidateNilAttestation(tt.attestation))
			} else {
				require.NoError(t, helpers.ValidateNilAttestation(tt.attestation))
			}
		})
	}
}

func TestValidateSlotTargetEpoch(t *testing.T) {
	tests := []struct {
		name        string
		attestation *ethpb.Attestation
		errString   string
	}{
		{
			name: "incorrect slot",
			attestation: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					Target: &ethpb.Checkpoint{Epoch: 1},
					Source: &ethpb.Checkpoint{},
				},
				AggregationBits: []byte{},
			},
			errString: "slot 0 does not match target epoch 1",
		},
		{
			name: "good attestation",
			attestation: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					Slot:   2 * params.BeaconConfig().SlotsPerEpoch,
					Target: &ethpb.Checkpoint{Epoch: 2},
					Source: &ethpb.Checkpoint{},
				},
				AggregationBits: []byte{},
			},
			errString: "",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			helpers.ClearCache()

			if tt.errString != "" {
				require.ErrorContains(t, tt.errString, helpers.ValidateSlotTargetEpoch(tt.attestation.Data))
			} else {
				require.NoError(t, helpers.ValidateSlotTargetEpoch(tt.attestation.Data))
			}
		})
	}
}
