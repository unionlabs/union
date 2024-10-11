package doublylinkedtree

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestNoVote_CanFindHead(t *testing.T) {
	f := setup(1, 1)
	f.justifiedBalances = make([]uint64, 16)
	ctx := context.Background()

	// The head should always start at the finalized block.
	r, err := f.Head(context.Background())
	require.NoError(t, err)
	if r != params.BeaconConfig().ZeroHash {
		t.Errorf("Incorrect head with genesis")
	}

	// Insert block 2 into the tree and verify head is at 2:
	//         0
	//        /
	//       2 <- head
	state, blkRoot, err := prepareForkchoiceState(context.Background(), 0, indexToHash(2), params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(2), r, "Incorrect head for with justified epoch at 1")

	// Insert block 1 into the tree and verify head is still at 2:
	//            0
	//           / \
	//  head -> 2  1
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(1), params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(2), r, "Incorrect head for with justified epoch at 1")

	// Insert block 3 into the tree and verify head is still at 2:
	//            0
	//           / \
	//  head -> 2  1
	//             |
	//             3
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(3), indexToHash(1), params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(2), r, "Incorrect head for with justified epoch at 1")

	// Insert block 4 into the tree and verify head is at 4:
	//            0
	//           / \
	//          2  1
	//          |  |
	//  head -> 4  3
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(4), indexToHash(2), params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(4), r, "Incorrect head for with justified epoch at 1")

	// Insert block 5 with justified epoch of 2, verify head is 5
	//            0
	//           / \
	//          2  1
	//          |  |
	//  head -> 4  3
	//          |
	//          5 <- justified epoch = 2
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(5), indexToHash(4), params.BeaconConfig().ZeroHash, 2, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(5), r, "Incorrect head for with justified epoch at 2")

	// Insert block 6 with justified epoch of 2, verify head is at 6.
	//            0
	//           / \
	//          2  1
	//          |  |
	//          4  3
	//          |
	//          5
	//          |
	//          6 <- head
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(6), indexToHash(5), params.BeaconConfig().ZeroHash, 2, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(6), r, "Incorrect head for with justified epoch at 2")
}
