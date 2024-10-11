package transition_test

import (
	"context"
	"sync"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestSkipSlotCache_OK(t *testing.T) {
	transition.SkipSlotCache.Enable()
	defer transition.SkipSlotCache.Disable()
	bState, privs := util.DeterministicGenesisState(t, params.MinimalSpecConfig().MinGenesisActiveValidatorCount)
	pbState, err := state_native.ProtobufBeaconStatePhase0(bState.ToProto())
	require.NoError(t, err)
	originalState, err := state_native.InitializeFromProtoPhase0(pbState)
	require.NoError(t, err)

	blkCfg := util.DefaultBlockGenConfig()
	blkCfg.NumAttestations = 1

	// First transition will be with an empty cache, so the cache becomes populated
	// with the state
	blk, err := util.GenerateFullBlock(bState, privs, blkCfg, originalState.Slot()+10)
	require.NoError(t, err)
	wsb, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	executedState, err := transition.ExecuteStateTransition(context.Background(), originalState, wsb)
	require.NoError(t, err, "Could not run state transition")
	require.Equal(t, true, executedState.Version() == version.Phase0)
	wsb, err = blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	bState, err = transition.ExecuteStateTransition(context.Background(), bState, wsb)
	require.NoError(t, err, "Could not process state transition")

	assert.DeepEqual(t, originalState.ToProto(), bState.ToProto(), "Skipped slots cache leads to different states")
}

func TestSkipSlotCache_ConcurrentMixup(t *testing.T) {
	bState, privs := util.DeterministicGenesisState(t, params.MinimalSpecConfig().MinGenesisActiveValidatorCount)
	pbState, err := state_native.ProtobufBeaconStatePhase0(bState.ToProto())
	require.NoError(t, err)
	originalState, err := state_native.InitializeFromProtoPhase0(pbState)
	require.NoError(t, err)

	blkCfg := util.DefaultBlockGenConfig()
	blkCfg.NumAttestations = 1

	transition.SkipSlotCache.Disable()

	// First transition will be with an empty cache, so the cache becomes populated
	// with the state
	blk, err := util.GenerateFullBlock(bState, privs, blkCfg, originalState.Slot()+10)
	require.NoError(t, err)
	wsb, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	executedState, err := transition.ExecuteStateTransition(context.Background(), originalState, wsb)
	require.NoError(t, err, "Could not run state transition")
	require.Equal(t, true, executedState.Version() == version.Phase0)

	// Create two shallow but different forks
	var s1, s0 state.BeaconState
	{
		blk, err := util.GenerateFullBlock(originalState.Copy(), privs, blkCfg, originalState.Slot()+10)
		require.NoError(t, err)
		copy(blk.Block.Body.Graffiti, "block 1")
		signature, err := util.BlockSignature(originalState, blk.Block, privs)
		require.NoError(t, err)
		blk.Signature = signature.Marshal()
		wsb, err := blocks.NewSignedBeaconBlock(blk)
		require.NoError(t, err)
		s1, err = transition.ExecuteStateTransition(context.Background(), originalState.Copy(), wsb)
		require.NoError(t, err, "Could not run state transition")
	}

	{
		blk, err := util.GenerateFullBlock(originalState.Copy(), privs, blkCfg, originalState.Slot()+10)
		require.NoError(t, err)
		copy(blk.Block.Body.Graffiti, "block 2")
		signature, err := util.BlockSignature(originalState, blk.Block, privs)
		require.NoError(t, err)
		blk.Signature = signature.Marshal()
		wsb, err := blocks.NewSignedBeaconBlock(blk)
		require.NoError(t, err)
		s0, err = transition.ExecuteStateTransition(context.Background(), originalState.Copy(), wsb)
		require.NoError(t, err, "Could not run state transition")
	}

	r1, err := s1.HashTreeRoot(context.Background())
	require.NoError(t, err)
	r2, err := s0.HashTreeRoot(context.Background())
	require.NoError(t, err)
	if r1 == r2 {
		t.Fatalf("need different starting states, got: %x", r1)
	}

	if s1.Slot() != s0.Slot() {
		t.Fatalf("expecting different chains, but states at same slot")
	}

	// prepare copies for both states
	var setups []state.BeaconState
	for i := uint64(0); i < 300; i++ {
		var st state.BeaconState
		if i%2 == 0 {
			st = s1
		} else {
			st = s0
		}
		setups = append(setups, st.Copy())
	}

	problemSlot := s1.Slot() + 2
	expected1, err := transition.ProcessSlots(context.Background(), s1.Copy(), problemSlot)
	require.NoError(t, err)
	expectedRoot1, err := expected1.HashTreeRoot(context.Background())
	require.NoError(t, err)
	t.Logf("chain 1 (even i) expected root %x at slot %d", expectedRoot1[:], problemSlot)

	tmp1, err := transition.ProcessSlots(context.Background(), expected1.Copy(), problemSlot+1)
	require.NoError(t, err)
	gotRoot := tmp1.StateRoots()[problemSlot]
	require.DeepEqual(t, expectedRoot1[:], gotRoot, "State roots for chain 1 are bad, expected root doesn't match")

	expected2, err := transition.ProcessSlots(context.Background(), s0.Copy(), problemSlot)
	require.NoError(t, err)
	expectedRoot2, err := expected2.HashTreeRoot(context.Background())
	require.NoError(t, err)
	t.Logf("chain 2 (odd i) expected root %x at slot %d", expectedRoot2[:], problemSlot)

	tmp2, err := transition.ProcessSlots(context.Background(), expected2.Copy(), problemSlot+1)
	require.NoError(t, err)
	gotRoot = tmp2.StateRoots()[problemSlot]
	require.DeepEqual(t, expectedRoot2[:], gotRoot, "State roots for chain 2 are bad, expected root doesn't match")

	var wg sync.WaitGroup
	wg.Add(len(setups))

	step := func(i int, setup state.BeaconState) {
		// go at least 1 past problemSlot, to ensure problem slot state root is available
		outState, err := transition.ProcessSlots(context.Background(), setup, problemSlot.Add(1+uint64(i))) // keep increasing, to hit and extend the cache
		require.NoError(t, err, "Could not process state transition")
		roots := outState.StateRoots()
		gotRoot := roots[problemSlot]
		if i%2 == 0 {
			assert.DeepEqual(t, expectedRoot1[:], gotRoot, "Unexpected root on chain 1")
		} else {
			assert.DeepEqual(t, expectedRoot2[:], gotRoot, "Unexpected root on chain 2")
		}
		wg.Done()
	}

	transition.SkipSlotCache.Enable()
	// now concurrently apply the blocks (alternating between states, and increasing skip slots)
	for i, setup := range setups {
		go step(i, setup)
	}
	// Wait for all transitions to finish
	wg.Wait()
}
