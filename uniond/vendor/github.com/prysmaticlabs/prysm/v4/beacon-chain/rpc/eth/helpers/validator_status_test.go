package helpers

import (
	"strconv"
	"testing"

	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	"github.com/prysmaticlabs/prysm/v4/proto/migration"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func Test_ValidatorStatus(t *testing.T) {
	farFutureEpoch := params.BeaconConfig().FarFutureEpoch

	type args struct {
		validator *ethpb.Validator
		epoch     primitives.Epoch
	}
	tests := []struct {
		name    string
		args    args
		want    validator.Status
		wantErr bool
	}{
		{
			name: "pending initialized",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:            farFutureEpoch,
					ActivationEligibilityEpoch: farFutureEpoch,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.Pending,
		},
		{
			name: "pending queued",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:            10,
					ActivationEligibilityEpoch: 2,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.Pending,
		},
		{
			name: "active ongoing",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch: 3,
					ExitEpoch:       farFutureEpoch,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.Active,
		},
		{
			name: "active slashed",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch: 3,
					ExitEpoch:       30,
					Slashed:         true,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.Active,
		},
		{
			name: "active exiting",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch: 3,
					ExitEpoch:       30,
					Slashed:         false,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.Active,
		},
		{
			name: "exited slashed",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:   3,
					ExitEpoch:         30,
					WithdrawableEpoch: 40,
					Slashed:           true,
				},
				epoch: primitives.Epoch(35),
			},
			want: validator.Exited,
		},
		{
			name: "exited unslashed",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:   3,
					ExitEpoch:         30,
					WithdrawableEpoch: 40,
					Slashed:           false,
				},
				epoch: primitives.Epoch(35),
			},
			want: validator.Exited,
		},
		{
			name: "withdrawal possible",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:   3,
					ExitEpoch:         30,
					WithdrawableEpoch: 40,
					EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance,
					Slashed:           false,
				},
				epoch: primitives.Epoch(45),
			},
			want: validator.Withdrawal,
		},
		{
			name: "withdrawal done",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:   3,
					ExitEpoch:         30,
					WithdrawableEpoch: 40,
					EffectiveBalance:  0,
					Slashed:           false,
				},
				epoch: primitives.Epoch(45),
			},
			want: validator.Withdrawal,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			readOnlyVal, err := state_native.NewValidator(migration.V1ValidatorToV1Alpha1(tt.args.validator))
			require.NoError(t, err)
			got, err := ValidatorStatus(readOnlyVal, tt.args.epoch)
			require.NoError(t, err)
			if got != tt.want {
				t.Errorf("validatorStatus() got = %v, want %v", got, tt.want)
			}
		})
	}
}

func Test_ValidatorSubStatus(t *testing.T) {
	farFutureEpoch := params.BeaconConfig().FarFutureEpoch

	type args struct {
		validator *ethpb.Validator
		epoch     primitives.Epoch
	}
	tests := []struct {
		name    string
		args    args
		want    validator.Status
		wantErr bool
	}{
		{
			name: "pending initialized",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:            farFutureEpoch,
					ActivationEligibilityEpoch: farFutureEpoch,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.PendingInitialized,
		},
		{
			name: "pending queued",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:            10,
					ActivationEligibilityEpoch: 2,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.PendingQueued,
		},
		{
			name: "active ongoing",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch: 3,
					ExitEpoch:       farFutureEpoch,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.ActiveOngoing,
		},
		{
			name: "active slashed",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch: 3,
					ExitEpoch:       30,
					Slashed:         true,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.ActiveSlashed,
		},
		{
			name: "active exiting",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch: 3,
					ExitEpoch:       30,
					Slashed:         false,
				},
				epoch: primitives.Epoch(5),
			},
			want: validator.ActiveExiting,
		},
		{
			name: "exited slashed",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:   3,
					ExitEpoch:         30,
					WithdrawableEpoch: 40,
					Slashed:           true,
				},
				epoch: primitives.Epoch(35),
			},
			want: validator.ExitedSlashed,
		},
		{
			name: "exited unslashed",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:   3,
					ExitEpoch:         30,
					WithdrawableEpoch: 40,
					Slashed:           false,
				},
				epoch: primitives.Epoch(35),
			},
			want: validator.ExitedUnslashed,
		},
		{
			name: "withdrawal possible",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:   3,
					ExitEpoch:         30,
					WithdrawableEpoch: 40,
					EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance,
					Slashed:           false,
				},
				epoch: primitives.Epoch(45),
			},
			want: validator.WithdrawalPossible,
		},
		{
			name: "withdrawal done",
			args: args{
				validator: &ethpb.Validator{
					ActivationEpoch:   3,
					ExitEpoch:         30,
					WithdrawableEpoch: 40,
					EffectiveBalance:  0,
					Slashed:           false,
				},
				epoch: primitives.Epoch(45),
			},
			want: validator.WithdrawalDone,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			readOnlyVal, err := state_native.NewValidator(migration.V1ValidatorToV1Alpha1(tt.args.validator))
			require.NoError(t, err)
			got, err := ValidatorSubStatus(readOnlyVal, tt.args.epoch)
			require.NoError(t, err)
			if got != tt.want {
				t.Errorf("validatorSubStatus() got = %v, want %v", got, tt.want)
			}
		})
	}
}

// This test verifies how many validator statuses have meaningful values.
// The first expected non-meaningful value will have x.String() equal to its numeric representation.
// This test assumes we start numbering from 0 and do not skip any values.
// Having a test like this allows us to use e.g. `if value < 10` for validity checks.
func TestNumberOfStatuses(t *testing.T) {
	lastValidEnumValue := 12
	x := ethpb.ValidatorStatus(lastValidEnumValue)
	assert.NotEqual(t, strconv.Itoa(lastValidEnumValue), x.String())
	x = ethpb.ValidatorStatus(lastValidEnumValue + 1)
	assert.Equal(t, strconv.Itoa(lastValidEnumValue+1), x.String())
}
