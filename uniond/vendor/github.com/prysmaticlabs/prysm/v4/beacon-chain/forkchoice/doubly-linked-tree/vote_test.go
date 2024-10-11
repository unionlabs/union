package doublylinkedtree

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestVotes_CanFindHead(t *testing.T) {
	f := setup(1, 1)
	f.justifiedBalances = []uint64{1, 1}
	ctx := context.Background()

	// The head should always start at the finalized block.
	r, err := f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, params.BeaconConfig().ZeroHash, r, "Incorrect head with genesis")

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

	// Add a vote to block 1 of the tree and verify head is switched to 1:
	//            0
	//           / \
	//          2  1 <- +vote, new head
	f.ProcessAttestation(context.Background(), []uint64{0}, indexToHash(1), 2)
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(1), r, "Incorrect head for with justified epoch at 1")

	// Add a vote to block 2 of the tree and verify head is switched to 2:
	//                     0
	//                    / \
	// vote, new head -> 2  1
	f.ProcessAttestation(context.Background(), []uint64{1}, indexToHash(2), 2)
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

	// Move validator 0's vote from 1 to 3 and verify head is still at 2:
	//            0
	//           / \
	//  head -> 2  1 <- old vote
	//             |
	//             3 <- new vote
	f.ProcessAttestation(context.Background(), []uint64{0}, indexToHash(3), 3)
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(2), r, "Incorrect head for with justified epoch at 1")

	// Move validator 1's vote from 2 to 1 and verify head is switched to 3:
	//               0
	//              / \
	// old vote -> 2  1 <- new vote
	//                |
	//                3 <- head
	f.ProcessAttestation(context.Background(), []uint64{1}, indexToHash(1), 3)
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(3), r, "Incorrect head for with justified epoch at 1")

	// Insert block 4 into the tree and verify head is at 4:
	//            0
	//           / \
	//          2  1
	//             |
	//             3
	//             |
	//             4 <- head
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(4), indexToHash(3), params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))

	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(4), r, "Incorrect head for with justified epoch at 1")

	// Insert block 5 with justified epoch 2, it becomes head
	//            0
	//           / \
	//          2  1
	//             |
	//             3
	//             |
	//             4
	//            /
	//           5 <- head, justified epoch = 2
	//
	// We set this node's slot to be 64 so that when pruning below we do not prune its child
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 2*params.BeaconConfig().SlotsPerEpoch, indexToHash(5), indexToHash(4), params.BeaconConfig().ZeroHash, 2, 2)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))

	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(5), r, "Incorrect head for with justified epoch at 2")

	// Insert block 6 with justified epoch 3: verify it's head
	//            0
	//           / \
	//          2  1
	//             |
	//             3
	//             |
	//             4
	//            / \
	//           5  6 <- head, justified epoch = 3
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(6), indexToHash(4), params.BeaconConfig().ZeroHash, 3, 2)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(6), r, "Incorrect head for with justified epoch at 3")

	// Moved 2 votes to block 5:
	f.ProcessAttestation(context.Background(), []uint64{0, 1}, indexToHash(5), 4)

	// Inset blocks 7 and 8
	// 6 should still be the head, even though 5 has all the votes.
	//            0
	//           / \
	//          2  1
	//             |
	//             3
	//             |
	//             4
	//            / \
	//           5  6 <- head
	//           |
	//           7
	//           |
	//           8
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(7), indexToHash(5), params.BeaconConfig().ZeroHash, 2, 2)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(8), indexToHash(7), params.BeaconConfig().ZeroHash, 2, 2)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(6), r, "Incorrect head for with justified epoch at 3")

	// Insert block 10 with justified epoch 3, it becomes head
	// Verify 10 is the head:
	//            0
	//           / \
	//          2  1
	//             |
	//             3
	//             |
	//             4
	//            / \
	//           5  6
	//           |
	//           7
	//           |
	//           8
	//           |
	//           10 <- head
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(10), indexToHash(8), params.BeaconConfig().ZeroHash, 3, 2)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(10), r, "Incorrect head for with justified epoch at 3")

	// Insert block 9 forking 10 verify it's head (lexicographic order)
	//             0
	//            / \
	//           2  1
	//              |
	//              3
	//              |
	//              4
	//             / \
	//            5  6
	//            |
	//            7
	//            |
	//            8
	//           / \
	//	    9  10
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(9), indexToHash(8), params.BeaconConfig().ZeroHash, 3, 2)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))

	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(9), r, "Incorrect head for with justified epoch at 3")

	// Move two votes for 10, verify it's head

	f.ProcessAttestation(context.Background(), []uint64{0, 1}, indexToHash(10), 5)
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(10), r, "Incorrect head for with justified epoch at 3")

	// Add 3 more validators to the system.
	f.justifiedBalances = []uint64{1, 1, 1, 1, 1}
	// The new validators voted for 9
	f.ProcessAttestation(context.Background(), []uint64{2, 3, 4}, indexToHash(9), 5)
	// The new head should be 9.
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(9), r, "Incorrect head for with justified epoch at 3")

	// Set the f.justifiedBalances of the last 2 validators to 0.
	f.justifiedBalances = []uint64{1, 1, 1, 0, 0}
	// The head should be back to 10.
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(10), r, "Incorrect head for with justified epoch at 3")

	// Set the f.justifiedBalances back to normal.
	f.justifiedBalances = []uint64{1, 1, 1, 1, 1}
	// The head should be back to 9.
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(9), r, "Incorrect head for with justified epoch at 3")

	// Remove the last 2 validators.
	f.justifiedBalances = []uint64{1, 1, 1}
	// The head should be back to 10.
	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(10), r, "Incorrect head for with justified epoch at 3")

	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(10), r, "Incorrect head for with justified epoch at 3")

	// Verify pruning above the prune threshold does prune:
	//          0
	//         / \
	//        2   1
	//            |
	//            3
	//            |
	//            4
	// -------pruned here ------
	//          5   6
	//          |
	//          7
	//          |
	//          8
	//         / \
	//        9  10
	f.store.finalizedCheckpoint.Root = indexToHash(5)
	require.NoError(t, f.store.prune(context.Background()))
	assert.Equal(t, 5, len(f.store.nodeByRoot), "Incorrect nodes length after prune")
	// we pruned artificially the justified root.
	f.store.justifiedCheckpoint.Root = indexToHash(5)

	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(10), r, "Incorrect head for with justified epoch at 3")

	// Insert new block 11 and verify head is at 11.
	//          5   6
	//          |
	//          7
	//          |
	//          8
	//         / \
	//        10  9
	//        |
	// head-> 11
	state, blkRoot, err = prepareForkchoiceState(context.Background(), 0, indexToHash(11), indexToHash(10), params.BeaconConfig().ZeroHash, 3, 2)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, state, blkRoot))

	r, err = f.Head(context.Background())
	require.NoError(t, err)
	assert.Equal(t, indexToHash(11), r, "Incorrect head for with justified epoch at 3")
}
