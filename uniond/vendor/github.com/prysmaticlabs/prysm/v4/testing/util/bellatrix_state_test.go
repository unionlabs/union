package util

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestDeterministicGenesisStateBellatrix(t *testing.T) {
	st, k := DeterministicGenesisStateBellatrix(t, params.BeaconConfig().MaxCommitteesPerSlot)
	require.Equal(t, params.BeaconConfig().MaxCommitteesPerSlot, uint64(len(k)))
	require.Equal(t, params.BeaconConfig().MaxCommitteesPerSlot, uint64(st.NumValidators()))
}

func TestGenesisBeaconStateBellatrix(t *testing.T) {
	ctx := context.Background()
	deposits, _, err := DeterministicDepositsAndKeys(params.BeaconConfig().MaxCommitteesPerSlot)
	require.NoError(t, err)
	eth1Data, err := DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	gt := uint64(10000)
	st, err := genesisBeaconStateBellatrix(ctx, deposits, gt, eth1Data)
	require.NoError(t, err)
	require.Equal(t, gt, st.GenesisTime())
	require.Equal(t, params.BeaconConfig().MaxCommitteesPerSlot, uint64(st.NumValidators()))
}
