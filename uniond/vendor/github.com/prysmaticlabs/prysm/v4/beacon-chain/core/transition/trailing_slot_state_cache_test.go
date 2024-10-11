package transition_test

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestTrailingSlotState_RoundTrip(t *testing.T) {
	ctx := context.Background()
	r := []byte{'a'}
	s := transition.NextSlotState(r, 0)
	require.Equal(t, nil, s)

	s, _ = util.DeterministicGenesisState(t, 1)
	require.NoError(t, transition.UpdateNextSlotCache(ctx, r, s))
	s = transition.NextSlotState(r, 1)
	require.Equal(t, primitives.Slot(1), s.Slot())

	lastRoot, lastState := transition.LastCachedState()
	require.DeepEqual(t, r, lastRoot)
	require.Equal(t, s.Slot(), lastState.Slot())

	require.NoError(t, transition.UpdateNextSlotCache(ctx, r, s))
	s = transition.NextSlotState(r, 2)
	require.Equal(t, primitives.Slot(2), s.Slot())

	lastRoot, lastState = transition.LastCachedState()
	require.DeepEqual(t, r, lastRoot)
	require.Equal(t, s.Slot(), lastState.Slot())
}

func TestTrailingSlotState_StateAdvancedBeyondRequest(t *testing.T) {
	ctx := context.Background()
	r := []byte{'a'}
	s := transition.NextSlotState(r, 0)
	require.Equal(t, nil, s)

	s, _ = util.DeterministicGenesisState(t, 1)
	assert.NoError(t, s.SetSlot(2))
	require.NoError(t, transition.UpdateNextSlotCache(ctx, r, s))
	s = transition.NextSlotState(r, 1)
	require.Equal(t, nil, s)
}
