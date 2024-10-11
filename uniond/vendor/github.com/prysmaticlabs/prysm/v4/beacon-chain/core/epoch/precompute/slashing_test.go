package precompute_test

import (
	"fmt"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch/precompute"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"google.golang.org/protobuf/proto"
)

func TestProcessSlashingsPrecompute_NotSlashedWithSlashedTrue(t *testing.T) {
	s, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Slot:       0,
		Validators: []*ethpb.Validator{{Slashed: true}},
		Balances:   []uint64{params.BeaconConfig().MaxEffectiveBalance},
		Slashings:  []uint64{0, 1e9},
	})
	require.NoError(t, err)
	pBal := &precompute.Balance{ActiveCurrentEpoch: params.BeaconConfig().MaxEffectiveBalance}
	require.NoError(t, precompute.ProcessSlashingsPrecompute(s, pBal))

	wanted := params.BeaconConfig().MaxEffectiveBalance
	assert.Equal(t, wanted, s.Balances()[0], "Unexpected slashed balance")
}

func TestProcessSlashingsPrecompute_NotSlashedWithSlashedFalse(t *testing.T) {
	s, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Slot:       0,
		Validators: []*ethpb.Validator{{}},
		Balances:   []uint64{params.BeaconConfig().MaxEffectiveBalance},
		Slashings:  []uint64{0, 1e9},
	})
	require.NoError(t, err)
	pBal := &precompute.Balance{ActiveCurrentEpoch: params.BeaconConfig().MaxEffectiveBalance}
	require.NoError(t, precompute.ProcessSlashingsPrecompute(s, pBal))

	wanted := params.BeaconConfig().MaxEffectiveBalance
	assert.Equal(t, wanted, s.Balances()[0], "Unexpected slashed balance")
}

func TestProcessSlashingsPrecompute_SlashedLess(t *testing.T) {
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
				Balances:  []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
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
				Balances:  []uint64{params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance, params.BeaconConfig().MaxEffectiveBalance},
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
			ab := uint64(0)
			for i, b := range tt.state.Balances {
				// Skip validator 0 since it's slashed
				if i == 0 {
					continue
				}
				ab += b
			}
			pBal := &precompute.Balance{ActiveCurrentEpoch: ab}

			original := proto.Clone(tt.state)
			state, err := state_native.InitializeFromProtoPhase0(tt.state)
			require.NoError(t, err)
			require.NoError(t, precompute.ProcessSlashingsPrecompute(state, pBal))
			assert.Equal(t, tt.want, state.Balances()[0], "ProcessSlashings({%v}) = newState; newState.Balances[0] = %d; wanted %d", original, state.Balances()[0])
		})
	}
}
