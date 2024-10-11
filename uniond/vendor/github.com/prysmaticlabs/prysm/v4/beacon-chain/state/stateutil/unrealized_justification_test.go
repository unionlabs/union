package stateutil

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestState_UnrealizedCheckpointBalances(t *testing.T) {
	validators := make([]*ethpb.Validator, params.BeaconConfig().MinGenesisActiveValidatorCount)
	targetFlag := params.BeaconConfig().TimelyTargetFlagIndex
	expectedActive := params.BeaconConfig().MinGenesisActiveValidatorCount * params.BeaconConfig().MaxEffectiveBalance

	balances := make([]uint64, params.BeaconConfig().MinGenesisActiveValidatorCount)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance,
		}
		balances[i] = params.BeaconConfig().MaxEffectiveBalance
	}
	cp := make([]byte, len(validators))
	pp := make([]byte, len(validators))

	t.Run("No one voted last two epochs", func(tt *testing.T) {
		active, previous, current, err := UnrealizedCheckpointBalances(cp, pp, validators, 0)
		require.NoError(tt, err)
		require.Equal(tt, expectedActive, active)
		require.Equal(tt, uint64(0), current)
		require.Equal(tt, uint64(0), previous)
	})

	t.Run("bad votes in last two epochs", func(tt *testing.T) {
		copy(cp, []byte{0xFF ^ (1 << targetFlag), 0xFF ^ (1 << targetFlag), 0xFF ^ (1 << targetFlag), 0x00})
		copy(pp, []byte{0x00, 0x00, 0x00, 0x00})
		active, previous, current, err := UnrealizedCheckpointBalances(cp, pp, validators, 1)
		require.NoError(tt, err)
		require.Equal(tt, expectedActive, active)
		require.Equal(tt, uint64(0), current)
		require.Equal(tt, uint64(0), previous)
	})

	t.Run("two votes in last epoch", func(tt *testing.T) {
		copy(cp, []byte{0xFF ^ (1 << targetFlag), 0xFF ^ (1 << targetFlag), 0xFF ^ (1 << targetFlag), 0x00, 1 << targetFlag, 1 << targetFlag})
		copy(pp, []byte{0x00, 0x00, 0x00, 0x00, 0xFF ^ (1 << targetFlag)})
		active, previous, current, err := UnrealizedCheckpointBalances(cp, pp, validators, 1)
		require.NoError(tt, err)
		require.Equal(tt, expectedActive, active)
		require.Equal(tt, 2*params.BeaconConfig().MaxEffectiveBalance, current)
		require.Equal(tt, uint64(0), previous)
	})

	t.Run("two votes in previous epoch", func(tt *testing.T) {
		copy(cp, []byte{0x00, 0x00, 0x00, 0x00, 0xFF ^ (1 << targetFlag), 0x00})
		copy(pp, []byte{0xFF ^ (1 << targetFlag), 0xFF ^ (1 << targetFlag), 0xFF ^ (1 << targetFlag), 0x00, 1 << targetFlag, 1 << targetFlag})
		active, previous, current, err := UnrealizedCheckpointBalances(cp, pp, validators, 1)
		require.NoError(tt, err)
		require.Equal(tt, expectedActive, active)
		require.Equal(tt, uint64(0), current)
		require.Equal(tt, 2*params.BeaconConfig().MaxEffectiveBalance, previous)
	})

	t.Run("votes in both epochs, decreased balance in first validator", func(tt *testing.T) {
		validators[0].EffectiveBalance = params.BeaconConfig().MaxEffectiveBalance - params.BeaconConfig().MinDepositAmount
		copy(cp, []byte{0xFF, 0xFF, 0x00, 0x00, 0xFF ^ (1 << targetFlag), 0})
		copy(pp, []byte{0xFF ^ (1 << targetFlag), 0xFF ^ (1 << targetFlag), 0xFF ^ (1 << targetFlag), 0x00, 0xFF, 0xFF})
		active, previous, current, err := UnrealizedCheckpointBalances(cp, pp, validators, 1)
		require.NoError(tt, err)
		expectedActive -= params.BeaconConfig().MinDepositAmount
		require.Equal(tt, expectedActive, active)
		require.Equal(tt, 2*params.BeaconConfig().MaxEffectiveBalance-params.BeaconConfig().MinDepositAmount, current)
		require.Equal(tt, 2*params.BeaconConfig().MaxEffectiveBalance, previous)
	})

	t.Run("slash a validator", func(tt *testing.T) {
		validators[1].Slashed = true
		active, previous, current, err := UnrealizedCheckpointBalances(cp, pp, validators, 1)
		require.NoError(tt, err)
		expectedActive -= params.BeaconConfig().MaxEffectiveBalance
		require.Equal(tt, expectedActive, active)
		require.Equal(tt, params.BeaconConfig().MaxEffectiveBalance-params.BeaconConfig().MinDepositAmount, current)
		require.Equal(tt, 2*params.BeaconConfig().MaxEffectiveBalance, previous)
	})
	t.Run("Exit a validator", func(tt *testing.T) {
		validators[4].ExitEpoch = 1
		active, previous, current, err := UnrealizedCheckpointBalances(cp, pp, validators, 2)
		require.NoError(tt, err)
		expectedActive -= params.BeaconConfig().MaxEffectiveBalance
		require.Equal(tt, expectedActive, active)
		require.Equal(tt, params.BeaconConfig().MaxEffectiveBalance-params.BeaconConfig().MinDepositAmount, current)
		require.Equal(tt, params.BeaconConfig().MaxEffectiveBalance, previous)
	})
}
