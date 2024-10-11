package state_native

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestState_UnrealizedCheckpointBalances(t *testing.T) {
	validators := make([]*ethpb.Validator, params.BeaconConfig().MinGenesisActiveValidatorCount)
	balances := make([]uint64, params.BeaconConfig().MinGenesisActiveValidatorCount)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance,
		}
		balances[i] = params.BeaconConfig().MaxEffectiveBalance
	}
	base := &ethpb.BeaconStateAltair{
		Slot:        2,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),

		Validators:                 validators,
		CurrentEpochParticipation:  make([]byte, params.BeaconConfig().MinGenesisActiveValidatorCount),
		PreviousEpochParticipation: make([]byte, params.BeaconConfig().MinGenesisActiveValidatorCount),
		Balances:                   balances,
	}
	state, err := InitializeFromProtoAltair(base)
	require.NoError(t, err)

	// No one voted in the last two epochs
	allActive := params.BeaconConfig().MinGenesisActiveValidatorCount * params.BeaconConfig().MaxEffectiveBalance
	active, previous, current, err := state.UnrealizedCheckpointBalances()
	require.NoError(t, err)
	require.Equal(t, allActive, active)
	require.Equal(t, uint64(0), current)
	require.Equal(t, uint64(0), previous)

	// Add some votes in the last two epochs:
	base.CurrentEpochParticipation[0] = 0xFF
	base.PreviousEpochParticipation[0] = 0xFF
	base.PreviousEpochParticipation[1] = 0xFF

	state, err = InitializeFromProtoAltair(base)
	require.NoError(t, err)
	active, previous, current, err = state.UnrealizedCheckpointBalances()
	require.NoError(t, err)
	require.Equal(t, allActive, active)
	require.Equal(t, params.BeaconConfig().MaxEffectiveBalance, current)
	require.Equal(t, 2*params.BeaconConfig().MaxEffectiveBalance, previous)

	// Slash some validators
	validators[0].Slashed = true
	state, err = InitializeFromProtoAltair(base)
	require.NoError(t, err)
	active, previous, current, err = state.UnrealizedCheckpointBalances()
	require.NoError(t, err)
	require.Equal(t, allActive-params.BeaconConfig().MaxEffectiveBalance, active)
	require.Equal(t, uint64(0), current)
	require.Equal(t, params.BeaconConfig().MaxEffectiveBalance, previous)

}
