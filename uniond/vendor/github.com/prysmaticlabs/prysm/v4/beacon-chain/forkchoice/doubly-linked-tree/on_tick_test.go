package doublylinkedtree

import (
	"context"
	"testing"

	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStore_NewSlot(t *testing.T) {
	ctx := context.Background()
	bj := [32]byte{'z'}

	type args struct {
		slot          primitives.Slot
		finalized     *forkchoicetypes.Checkpoint
		justified     *forkchoicetypes.Checkpoint
		bestJustified *forkchoicetypes.Checkpoint
		shouldEqual   bool
	}
	tests := []struct {
		name string
		args args
	}{
		{
			name: "Not epoch boundary. No change",
			args: args{
				slot:          params.BeaconConfig().SlotsPerEpoch + 1,
				finalized:     &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'a'}},
				justified:     &forkchoicetypes.Checkpoint{Epoch: 2, Root: [32]byte{'b'}},
				bestJustified: &forkchoicetypes.Checkpoint{Epoch: 3, Root: bj},
				shouldEqual:   false,
			},
		},
		{
			name: "Justified higher than best justified. No change",
			args: args{
				slot:          params.BeaconConfig().SlotsPerEpoch,
				finalized:     &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'a'}},
				justified:     &forkchoicetypes.Checkpoint{Epoch: 3, Root: [32]byte{'b'}},
				bestJustified: &forkchoicetypes.Checkpoint{Epoch: 2, Root: bj},
				shouldEqual:   false,
			},
		},
		{
			name: "Best justified not on the same chain as finalized. No change",
			args: args{
				slot:          params.BeaconConfig().SlotsPerEpoch,
				finalized:     &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'a'}},
				justified:     &forkchoicetypes.Checkpoint{Epoch: 2, Root: [32]byte{'b'}},
				bestJustified: &forkchoicetypes.Checkpoint{Epoch: 3, Root: [32]byte{'d'}},
				shouldEqual:   false,
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			f := setup(test.args.justified.Epoch, test.args.finalized.Epoch)
			state, blkRoot, err := prepareForkchoiceState(ctx, 0, [32]byte{}, [32]byte{}, [32]byte{}, 0, 0)
			require.NoError(t, err)
			require.NoError(t, f.InsertNode(ctx, state, blkRoot)) // genesis
			state, blkRoot, err = prepareForkchoiceState(ctx, 32, [32]byte{'a'}, [32]byte{}, [32]byte{}, 0, 0)
			require.NoError(t, err)
			require.NoError(t, f.InsertNode(ctx, state, blkRoot)) // finalized
			state, blkRoot, err = prepareForkchoiceState(ctx, 64, [32]byte{'b'}, [32]byte{'a'}, [32]byte{}, 0, 0)
			require.NoError(t, err)
			require.NoError(t, f.InsertNode(ctx, state, blkRoot)) // justified
			state, blkRoot, err = prepareForkchoiceState(ctx, 96, bj, [32]byte{'a'}, [32]byte{}, 0, 0)
			require.NoError(t, err)
			require.NoError(t, f.InsertNode(ctx, state, blkRoot)) // best justified
			state, blkRoot, err = prepareForkchoiceState(ctx, 97, [32]byte{'d'}, [32]byte{}, [32]byte{}, 0, 0)
			require.NoError(t, err)
			require.NoError(t, f.InsertNode(ctx, state, blkRoot)) // bad

			require.NoError(t, f.UpdateFinalizedCheckpoint(test.args.finalized))
			require.NoError(t, f.UpdateJustifiedCheckpoint(ctx, test.args.justified))

			require.NoError(t, f.NewSlot(ctx, test.args.slot))
			bcp := test.args.bestJustified
			if test.args.shouldEqual {
				cp := f.JustifiedCheckpoint()
				require.Equal(t, bcp.Epoch, cp.Epoch)
				require.Equal(t, bcp.Root, cp.Root)
			} else {
				cp := f.JustifiedCheckpoint()
				epochsEqual := bcp.Epoch == cp.Epoch
				rootsEqual := bcp.Root == cp.Root
				require.Equal(t, false, epochsEqual && rootsEqual)
			}
		})
	}
}
