package doublylinkedtree

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestLastRoot(t *testing.T) {
	f := setup(0, 0)
	ctx := context.Background()

	st, root, err := prepareForkchoiceState(ctx, 1, [32]byte{'1'}, params.BeaconConfig().ZeroHash, [32]byte{'1'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, root))
	st, root, err = prepareForkchoiceState(ctx, 2, [32]byte{'2'}, [32]byte{'1'}, [32]byte{'2'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, root))
	st, root, err = prepareForkchoiceState(ctx, 3, [32]byte{'3'}, [32]byte{'1'}, [32]byte{'3'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, root))
	st, root, err = prepareForkchoiceState(ctx, 32, [32]byte{'4'}, [32]byte{'3'}, [32]byte{'4'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, root))
	st, root, err = prepareForkchoiceState(ctx, 33, [32]byte{'5'}, [32]byte{'2'}, [32]byte{'5'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, root))
	st, root, err = prepareForkchoiceState(ctx, 34, [32]byte{'6'}, [32]byte{'5'}, [32]byte{'6'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, root))
	headNode, _ := f.store.nodeByRoot[[32]byte{'6'}]
	f.store.headNode = headNode
	require.Equal(t, [32]byte{'6'}, f.store.headNode.root)
	require.Equal(t, [32]byte{'2'}, f.LastRoot(0))
	require.Equal(t, [32]byte{'6'}, f.LastRoot(1))
	require.Equal(t, [32]byte{'6'}, f.LastRoot(2))
}
