package state_native

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func FuzzMultiValueBalances(f *testing.F) {
	resetFn := features.InitWithReset(&features.Flags{
		EnableExperimentalState: true,
	})
	defer resetFn()

	bals := make([]uint64, 65536)
	firstState, err := InitializeFromProtoPhase0(&ethpb.BeaconState{Balances: bals})
	require.NoError(f, err)

	f.Fuzz(func(t *testing.T, index uint16, value uint64) {
		secondState := firstState
		// there's a 25% chance we will copy the state
		copyState := index%4 == 0
		if copyState {
			secondState = firstState.Copy()
		}
		if index%2 == 0 {
			// update existing balance

			oldValue, err := firstState.BalanceAtIndex(primitives.ValidatorIndex(index))
			require.NoError(t, err)

			require.NoError(t, secondState.UpdateBalancesAtIndex(primitives.ValidatorIndex(index), value))

			firstValue, err := firstState.BalanceAtIndex(primitives.ValidatorIndex(index))
			require.NoError(t, err)
			secondValue, err := secondState.BalanceAtIndex(primitives.ValidatorIndex(index))
			require.NoError(t, err)
			if copyState {
				require.Equal(t, oldValue, firstValue)
				require.Equal(t, value, secondValue)
			} else {
				require.Equal(t, value, firstValue)
				require.Equal(t, value, secondValue)
			}
		} else {
			// append new balance

			firstLength := firstState.BalancesLength()

			require.NoError(t, secondState.AppendBalance(value))

			if copyState {
				require.Equal(t, firstLength, secondState.BalancesLength())
				v, err := firstState.BalanceAtIndex(primitives.ValidatorIndex(firstLength - 1))
				require.NoError(t, err)
				require.Equal(t, value, v)
				v, err = secondState.BalanceAtIndex(primitives.ValidatorIndex(secondState.BalancesLength() - 1))
				require.NoError(t, err)
				require.Equal(t, value, v)
			} else {
				require.Equal(t, firstLength+1, secondState.BalancesLength())
				v, err := secondState.BalanceAtIndex(primitives.ValidatorIndex(secondState.BalancesLength() - 1))
				require.NoError(t, err)
				require.Equal(t, value, v)
			}
		}
	})
}
