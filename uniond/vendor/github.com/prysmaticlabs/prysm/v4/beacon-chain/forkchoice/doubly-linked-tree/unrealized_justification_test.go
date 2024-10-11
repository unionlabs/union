package doublylinkedtree

import (
	"context"
	"testing"

	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStore_SetUnrealizedEpochs(t *testing.T) {
	f := setup(1, 1)
	ctx := context.Background()
	state, blkRoot, err := prepareForkchoiceState(ctx, 100, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 101, [32]byte{'b'}, [32]byte{'a'}, [32]byte{'B'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 102, [32]byte{'c'}, [32]byte{'b'}, [32]byte{'C'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	require.Equal(t, primitives.Epoch(1), f.store.nodeByRoot[[32]byte{'b'}].unrealizedJustifiedEpoch)
	require.Equal(t, primitives.Epoch(1), f.store.nodeByRoot[[32]byte{'b'}].unrealizedFinalizedEpoch)
	require.NoError(t, f.store.setUnrealizedJustifiedEpoch([32]byte{'b'}, 2))
	require.NoError(t, f.store.setUnrealizedFinalizedEpoch([32]byte{'b'}, 2))
	require.Equal(t, primitives.Epoch(2), f.store.nodeByRoot[[32]byte{'b'}].unrealizedJustifiedEpoch)
	require.Equal(t, primitives.Epoch(2), f.store.nodeByRoot[[32]byte{'b'}].unrealizedFinalizedEpoch)

	require.ErrorIs(t, errInvalidUnrealizedJustifiedEpoch, f.store.setUnrealizedJustifiedEpoch([32]byte{'b'}, 0))
	require.ErrorIs(t, errInvalidUnrealizedFinalizedEpoch, f.store.setUnrealizedFinalizedEpoch([32]byte{'b'}, 0))
}

func TestStore_UpdateUnrealizedCheckpoints(t *testing.T) {
	f := setup(1, 1)
	ctx := context.Background()
	state, blkRoot, err := prepareForkchoiceState(ctx, 100, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 101, [32]byte{'b'}, [32]byte{'a'}, [32]byte{'B'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 102, [32]byte{'c'}, [32]byte{'b'}, [32]byte{'C'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))

}

// Epoch 2    |   Epoch 3
//
//	    |
//	  C |
//	/   |
//
// A <-- B    |
//
//	\   |
//	  ---- D
//
// B is the first block that justifies A.
func TestStore_LongFork(t *testing.T) {
	f := setup(1, 1)
	ctx := context.Background()
	state, blkRoot, err := prepareForkchoiceState(ctx, 100, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 101, [32]byte{'b'}, [32]byte{'a'}, [32]byte{'B'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	require.NoError(t, f.store.setUnrealizedJustifiedEpoch([32]byte{'b'}, 2))
	state, blkRoot, err = prepareForkchoiceState(ctx, 102, [32]byte{'c'}, [32]byte{'b'}, [32]byte{'C'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	require.NoError(t, f.store.setUnrealizedJustifiedEpoch([32]byte{'c'}, 2))

	// Add an attestation to c, it is head
	f.ProcessAttestation(ctx, []uint64{0}, [32]byte{'c'}, 1)
	f.justifiedBalances = []uint64{100}
	headRoot, err := f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'c'}, headRoot)

	// D is head even though its weight is lower.
	ha := [32]byte{'a'}
	state, blkRoot, err = prepareForkchoiceState(ctx, 103, [32]byte{'d'}, [32]byte{'b'}, [32]byte{'D'}, 2, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	require.NoError(t, f.UpdateJustifiedCheckpoint(ctx, &forkchoicetypes.Checkpoint{Epoch: 2, Root: ha}))
	headRoot, err = f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'d'}, headRoot)
	require.Equal(t, uint64(0), f.store.nodeByRoot[[32]byte{'d'}].weight)
	require.Equal(t, uint64(100), f.store.nodeByRoot[[32]byte{'c'}].weight)

	// Update unrealized justification, c becomes head
	require.NoError(t, f.updateUnrealizedCheckpoints(ctx))
	headRoot, err = f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'c'}, headRoot)
}

//	Epoch 1                Epoch 2               Epoch 3
//	                |                      |
//	                |                      |
//
// A <-- B <-- C <-- D <-- E <-- F <-- G <-- H |
//
//	|        \             |
//	|         --------------- I
//	|                      |
//
// E justifies A. G justifies E.
func TestStore_NoDeadLock(t *testing.T) {
	f := setup(0, 0)
	ctx := context.Background()

	// Epoch 1 blocks
	state, blkRoot, err := prepareForkchoiceState(ctx, 100, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 101, [32]byte{'b'}, [32]byte{'a'}, [32]byte{'B'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 102, [32]byte{'c'}, [32]byte{'b'}, [32]byte{'C'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 103, [32]byte{'d'}, [32]byte{'c'}, [32]byte{'D'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))

	// Epoch 2 Blocks
	state, blkRoot, err = prepareForkchoiceState(ctx, 104, [32]byte{'e'}, [32]byte{'d'}, [32]byte{'E'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	require.NoError(t, f.store.setUnrealizedJustifiedEpoch([32]byte{'e'}, 1))
	state, blkRoot, err = prepareForkchoiceState(ctx, 105, [32]byte{'f'}, [32]byte{'e'}, [32]byte{'F'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	require.NoError(t, f.store.setUnrealizedJustifiedEpoch([32]byte{'f'}, 1))
	state, blkRoot, err = prepareForkchoiceState(ctx, 106, [32]byte{'g'}, [32]byte{'f'}, [32]byte{'G'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	require.NoError(t, f.store.setUnrealizedJustifiedEpoch([32]byte{'g'}, 2))
	require.NoError(t, f.store.setUnrealizedFinalizedEpoch([32]byte{'g'}, 1))
	f.store.unrealizedJustifiedCheckpoint = &forkchoicetypes.Checkpoint{Epoch: 2}
	f.store.unrealizedFinalizedCheckpoint = &forkchoicetypes.Checkpoint{Epoch: 1}
	state, blkRoot, err = prepareForkchoiceState(ctx, 107, [32]byte{'h'}, [32]byte{'g'}, [32]byte{'H'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	require.NoError(t, f.store.setUnrealizedJustifiedEpoch([32]byte{'h'}, 2))
	require.NoError(t, f.store.setUnrealizedFinalizedEpoch([32]byte{'h'}, 1))
	// Add an attestation for h
	f.ProcessAttestation(ctx, []uint64{0}, [32]byte{'h'}, 1)

	// Epoch 3
	// Current Head is H
	f.justifiedBalances = []uint64{100}
	headRoot, err := f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'h'}, headRoot)
	require.Equal(t, primitives.Epoch(0), f.JustifiedCheckpoint().Epoch)

	// Insert Block I, it becomes Head
	hr := [32]byte{'i'}
	state, blkRoot, err = prepareForkchoiceState(ctx, 108, hr, [32]byte{'f'}, [32]byte{'I'}, 1, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	ha := [32]byte{'a'}
	require.NoError(t, f.UpdateJustifiedCheckpoint(ctx, &forkchoicetypes.Checkpoint{Epoch: 1, Root: ha}))
	headRoot, err = f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'i'}, headRoot)
	require.Equal(t, primitives.Epoch(1), f.JustifiedCheckpoint().Epoch)
	require.Equal(t, primitives.Epoch(0), f.FinalizedCheckpoint().Epoch)

	// Realized Justified checkpoints, H becomes head
	require.NoError(t, f.updateUnrealizedCheckpoints(ctx))
	headRoot, err = f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'h'}, headRoot)
	require.Equal(t, primitives.Epoch(2), f.JustifiedCheckpoint().Epoch)
	require.Equal(t, primitives.Epoch(1), f.FinalizedCheckpoint().Epoch)
}

//	  Epoch  2       |         Epoch 3
//	                 |
//	            -- D (late)
//	           /     |
//	A <- B <- C      |
//	           \     |
//	            -- -- -- E <- F <- G <- H
//	                 |
//
// D justifies and comes late.
func TestStore_ForkNextEpoch(t *testing.T) {
	f := setup(1, 0)
	ctx := context.Background()

	// Epoch 1 blocks (D does not arrive)
	state, blkRoot, err := prepareForkchoiceState(ctx, 92, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 1, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 93, [32]byte{'b'}, [32]byte{'a'}, [32]byte{'B'}, 1, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 94, [32]byte{'c'}, [32]byte{'b'}, [32]byte{'C'}, 1, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))

	// Epoch 2 blocks
	state, blkRoot, err = prepareForkchoiceState(ctx, 96, [32]byte{'e'}, [32]byte{'c'}, [32]byte{'E'}, 1, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 97, [32]byte{'f'}, [32]byte{'e'}, [32]byte{'F'}, 1, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 98, [32]byte{'g'}, [32]byte{'f'}, [32]byte{'G'}, 1, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 99, [32]byte{'h'}, [32]byte{'g'}, [32]byte{'H'}, 1, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))

	// Insert an attestation to H, H is head
	f.ProcessAttestation(ctx, []uint64{0}, [32]byte{'h'}, 1)
	f.justifiedBalances = []uint64{100}
	headRoot, err := f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'h'}, headRoot)
	require.Equal(t, primitives.Epoch(1), f.JustifiedCheckpoint().Epoch)

	// D arrives late, D is head
	state, blkRoot, err = prepareForkchoiceState(ctx, 95, [32]byte{'d'}, [32]byte{'c'}, [32]byte{'D'}, 1, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	require.NoError(t, f.store.setUnrealizedJustifiedEpoch([32]byte{'d'}, 2))
	f.store.unrealizedJustifiedCheckpoint = &forkchoicetypes.Checkpoint{Epoch: 2}
	require.NoError(t, f.updateUnrealizedCheckpoints(ctx))
	headRoot, err = f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'d'}, headRoot)
	require.Equal(t, primitives.Epoch(2), f.JustifiedCheckpoint().Epoch)
	require.Equal(t, uint64(0), f.store.nodeByRoot[[32]byte{'d'}].weight)
	require.Equal(t, uint64(100), f.store.nodeByRoot[[32]byte{'h'}].weight)
	// Set current epoch to 3, and H's unrealized checkpoint. Check it's head
	driftGenesisTime(f, 99, 0)
	require.NoError(t, f.store.setUnrealizedJustifiedEpoch([32]byte{'h'}, 2))
	headRoot, err = f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'h'}, headRoot)
	require.Equal(t, primitives.Epoch(2), f.JustifiedCheckpoint().Epoch)
	require.Equal(t, uint64(0), f.store.nodeByRoot[[32]byte{'d'}].weight)
	require.Equal(t, uint64(100), f.store.nodeByRoot[[32]byte{'h'}].weight)
}

func TestStore_PullTips_Heuristics(t *testing.T) {
	ctx := context.Background()
	t.Run("Current epoch is justified", func(tt *testing.T) {
		f := setup(1, 1)
		st, root, err := prepareForkchoiceState(ctx, 65, [32]byte{'p'}, [32]byte{}, [32]byte{}, 1, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		f.store.nodeByRoot[[32]byte{'p'}].unrealizedJustifiedEpoch = primitives.Epoch(2)
		driftGenesisTime(f, 66, 0)

		st, root, err = prepareForkchoiceState(ctx, 66, [32]byte{'h'}, [32]byte{'p'}, [32]byte{}, 1, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		require.Equal(tt, primitives.Epoch(2), f.store.nodeByRoot[[32]byte{'h'}].unrealizedJustifiedEpoch)
		require.Equal(tt, primitives.Epoch(1), f.store.nodeByRoot[[32]byte{'h'}].unrealizedFinalizedEpoch)
	})

	t.Run("Previous Epoch is justified and too early for current", func(tt *testing.T) {
		f := setup(1, 1)
		st, root, err := prepareForkchoiceState(ctx, 95, [32]byte{'p'}, [32]byte{}, [32]byte{}, 1, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		f.store.nodeByRoot[[32]byte{'p'}].unrealizedJustifiedEpoch = primitives.Epoch(2)
		driftGenesisTime(f, 96, 0)

		st, root, err = prepareForkchoiceState(ctx, 96, [32]byte{'h'}, [32]byte{'p'}, [32]byte{}, 1, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		require.Equal(tt, primitives.Epoch(2), f.store.nodeByRoot[[32]byte{'h'}].unrealizedJustifiedEpoch)
		require.Equal(tt, primitives.Epoch(1), f.store.nodeByRoot[[32]byte{'h'}].unrealizedFinalizedEpoch)
	})
	t.Run("Previous Epoch is justified and not too early for current", func(tt *testing.T) {
		f := setup(1, 1)
		st, root, err := prepareForkchoiceState(ctx, 95, [32]byte{'p'}, [32]byte{}, [32]byte{}, 1, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		f.store.nodeByRoot[[32]byte{'p'}].unrealizedJustifiedEpoch = primitives.Epoch(2)
		driftGenesisTime(f, 127, 0)

		st, root, err = prepareForkchoiceState(ctx, 127, [32]byte{'h'}, [32]byte{'p'}, [32]byte{}, 1, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		// Check that the justification point is not the parent's.
		// This test checks that the heuristics in pullTips did not apply and
		// the test continues to compute a bogus unrealized
		// justification
		require.Equal(tt, primitives.Epoch(1), f.store.nodeByRoot[[32]byte{'h'}].unrealizedJustifiedEpoch)
	})
	t.Run("Block from previous Epoch", func(tt *testing.T) {
		f := setup(1, 1)
		st, root, err := prepareForkchoiceState(ctx, 94, [32]byte{'p'}, [32]byte{}, [32]byte{}, 1, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		f.store.nodeByRoot[[32]byte{'p'}].unrealizedJustifiedEpoch = primitives.Epoch(2)
		driftGenesisTime(f, 96, 0)

		st, root, err = prepareForkchoiceState(ctx, 95, [32]byte{'h'}, [32]byte{'p'}, [32]byte{}, 1, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		// Check that the justification point is not the parent's.
		// This test checks that the heuristics in pullTips did not apply and
		// the test continues to compute a bogus unrealized
		// justification
		require.Equal(tt, primitives.Epoch(1), f.store.nodeByRoot[[32]byte{'h'}].unrealizedJustifiedEpoch)
	})
	t.Run("Previous Epoch is not justified", func(tt *testing.T) {
		f := setup(1, 1)
		st, root, err := prepareForkchoiceState(ctx, 128, [32]byte{'p'}, [32]byte{}, [32]byte{}, 2, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		driftGenesisTime(f, 129, 0)

		st, root, err = prepareForkchoiceState(ctx, 129, [32]byte{'h'}, [32]byte{'p'}, [32]byte{}, 2, 1)
		require.NoError(tt, err)
		require.NoError(tt, f.InsertNode(ctx, st, root))
		// Check that the justification point is not the parent's.
		// This test checks that the heuristics in pullTips did not apply and
		// the test continues to compute a bogus unrealized
		// justification
		require.Equal(tt, primitives.Epoch(2), f.store.nodeByRoot[[32]byte{'h'}].unrealizedJustifiedEpoch)
	})
}
