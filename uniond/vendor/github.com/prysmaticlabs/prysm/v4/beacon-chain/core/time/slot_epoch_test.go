package time_test

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestSlotToEpoch_OK(t *testing.T) {
	tests := []struct {
		slot  primitives.Slot
		epoch primitives.Epoch
	}{
		{slot: 0, epoch: 0},
		{slot: 50, epoch: 1},
		{slot: 64, epoch: 2},
		{slot: 128, epoch: 4},
		{slot: 200, epoch: 6},
	}
	for _, tt := range tests {
		assert.Equal(t, tt.epoch, slots.ToEpoch(tt.slot), "ToEpoch(%d)", tt.slot)
	}
}

func TestCurrentEpoch_OK(t *testing.T) {
	tests := []struct {
		slot  primitives.Slot
		epoch primitives.Epoch
	}{
		{slot: 0, epoch: 0},
		{slot: 50, epoch: 1},
		{slot: 64, epoch: 2},
		{slot: 128, epoch: 4},
		{slot: 200, epoch: 6},
	}
	for _, tt := range tests {
		st, err := state_native.InitializeFromProtoPhase0(&eth.BeaconState{Slot: tt.slot})
		require.NoError(t, err)
		assert.Equal(t, tt.epoch, time.CurrentEpoch(st), "ActiveCurrentEpoch(%d)", st.Slot())
	}
}

func TestPrevEpoch_OK(t *testing.T) {
	tests := []struct {
		slot  primitives.Slot
		epoch primitives.Epoch
	}{
		{slot: 0, epoch: 0},
		{slot: 0 + params.BeaconConfig().SlotsPerEpoch + 1, epoch: 0},
		{slot: 2 * params.BeaconConfig().SlotsPerEpoch, epoch: 1},
	}
	for _, tt := range tests {
		st, err := state_native.InitializeFromProtoPhase0(&eth.BeaconState{Slot: tt.slot})
		require.NoError(t, err)
		assert.Equal(t, tt.epoch, time.PrevEpoch(st), "ActivePrevEpoch(%d)", st.Slot())
	}
}

func TestNextEpoch_OK(t *testing.T) {
	tests := []struct {
		slot  primitives.Slot
		epoch primitives.Epoch
	}{
		{slot: 0, epoch: primitives.Epoch(0/params.BeaconConfig().SlotsPerEpoch + 1)},
		{slot: 50, epoch: primitives.Epoch(0/params.BeaconConfig().SlotsPerEpoch + 2)},
		{slot: 64, epoch: primitives.Epoch(64/params.BeaconConfig().SlotsPerEpoch + 1)},
		{slot: 128, epoch: primitives.Epoch(128/params.BeaconConfig().SlotsPerEpoch + 1)},
		{slot: 200, epoch: primitives.Epoch(200/params.BeaconConfig().SlotsPerEpoch + 1)},
	}
	for _, tt := range tests {
		st, err := state_native.InitializeFromProtoPhase0(&eth.BeaconState{Slot: tt.slot})
		require.NoError(t, err)
		assert.Equal(t, tt.epoch, time.NextEpoch(st), "NextEpoch(%d)", st.Slot())
	}
}

func TestCanUpgradeToAltair(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	bc := params.BeaconConfig()
	bc.AltairForkEpoch = 5
	params.OverrideBeaconConfig(bc)
	tests := []struct {
		name string
		slot primitives.Slot
		want bool
	}{
		{
			name: "not epoch start",
			slot: 1,
			want: false,
		},
		{
			name: "not altair epoch",
			slot: params.BeaconConfig().SlotsPerEpoch,
			want: false,
		},
		{
			name: "altair epoch",
			slot: primitives.Slot(params.BeaconConfig().AltairForkEpoch) * params.BeaconConfig().SlotsPerEpoch,
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := time.CanUpgradeToAltair(tt.slot); got != tt.want {
				t.Errorf("canUpgradeToAltair() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestCanUpgradeBellatrix(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	bc := params.BeaconConfig()
	bc.BellatrixForkEpoch = 5
	params.OverrideBeaconConfig(bc)
	tests := []struct {
		name string
		slot primitives.Slot
		want bool
	}{
		{
			name: "not epoch start",
			slot: 1,
			want: false,
		},
		{
			name: "not bellatrix epoch",
			slot: params.BeaconConfig().SlotsPerEpoch,
			want: false,
		},
		{
			name: "bellatrix epoch",
			slot: primitives.Slot(params.BeaconConfig().BellatrixForkEpoch) * params.BeaconConfig().SlotsPerEpoch,
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := time.CanUpgradeToBellatrix(tt.slot); got != tt.want {
				t.Errorf("CanUpgradeToBellatrix() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestCanProcessEpoch_TrueOnEpochsLastSlot(t *testing.T) {
	tests := []struct {
		slot            primitives.Slot
		canProcessEpoch bool
	}{
		{
			slot:            1,
			canProcessEpoch: false,
		}, {
			slot:            63,
			canProcessEpoch: true,
		},
		{
			slot:            64,
			canProcessEpoch: false,
		}, {
			slot:            127,
			canProcessEpoch: true,
		}, {
			slot:            1000000000,
			canProcessEpoch: false,
		},
	}

	for _, tt := range tests {
		b := &eth.BeaconState{Slot: tt.slot}
		s, err := state_native.InitializeFromProtoPhase0(b)
		require.NoError(t, err)
		assert.Equal(t, tt.canProcessEpoch, time.CanProcessEpoch(s), "CanProcessEpoch(%d)", tt.slot)
	}
}

func TestAltairCompatible(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig()
	cfg.AltairForkEpoch = 1
	cfg.BellatrixForkEpoch = 2
	params.OverrideBeaconConfig(cfg)

	type args struct {
		s state.BeaconState
		e primitives.Epoch
	}
	tests := []struct {
		name string
		args args
		want bool
	}{
		{
			name: "phase0 state",
			args: args{
				s: func() state.BeaconState {
					st, _ := util.DeterministicGenesisState(t, 1)
					return st
				}(),
			},
			want: false,
		},
		{
			name: "altair state, altair epoch",
			args: args{
				s: func() state.BeaconState {
					st, _ := util.DeterministicGenesisStateAltair(t, 1)
					return st
				}(),
				e: params.BeaconConfig().AltairForkEpoch,
			},
			want: true,
		},
		{
			name: "bellatrix state, bellatrix epoch",
			args: args{
				s: func() state.BeaconState {
					st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
					return st
				}(),
				e: params.BeaconConfig().BellatrixForkEpoch,
			},
			want: true,
		},
		{
			name: "bellatrix state, altair epoch",
			args: args{
				s: func() state.BeaconState {
					st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
					return st
				}(),
				e: params.BeaconConfig().AltairForkEpoch,
			},
			want: true,
		},
		{
			name: "bellatrix state, phase0 epoch",
			args: args{
				s: func() state.BeaconState {
					st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
					return st
				}(),
			},
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := time.HigherEqualThanAltairVersionAndEpoch(tt.args.s, tt.args.e); got != tt.want {
				t.Errorf("HigherEqualThanAltairVersionAndEpoch() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestCanUpgradeToCapella(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	bc := params.BeaconConfig()
	bc.CapellaForkEpoch = 5
	params.OverrideBeaconConfig(bc)
	tests := []struct {
		name string
		slot primitives.Slot
		want bool
	}{
		{
			name: "not epoch start",
			slot: 1,
			want: false,
		},
		{
			name: "not capella epoch",
			slot: params.BeaconConfig().SlotsPerEpoch,
			want: false,
		},
		{
			name: "capella epoch",
			slot: primitives.Slot(params.BeaconConfig().CapellaForkEpoch) * params.BeaconConfig().SlotsPerEpoch,
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := time.CanUpgradeToCapella(tt.slot); got != tt.want {
				t.Errorf("CanUpgradeToCapella() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestCanUpgradeToDeneb(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	bc := params.BeaconConfig()
	bc.DenebForkEpoch = 5
	params.OverrideBeaconConfig(bc)
	tests := []struct {
		name string
		slot primitives.Slot
		want bool
	}{
		{
			name: "not epoch start",
			slot: 1,
			want: false,
		},
		{
			name: "not deneb epoch",
			slot: params.BeaconConfig().SlotsPerEpoch,
			want: false,
		},
		{
			name: "deneb epoch",
			slot: primitives.Slot(params.BeaconConfig().DenebForkEpoch) * params.BeaconConfig().SlotsPerEpoch,
			want: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := time.CanUpgradeToDeneb(tt.slot); got != tt.want {
				t.Errorf("CanUpgradeToDeneb() = %v, want %v", got, tt.want)
			}
		})
	}
}
