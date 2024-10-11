package doublylinkedtree

import (
	"context"
	"encoding/binary"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

// prepareForkchoiceState prepares a beacon State with the given data to mock
// insert into forkchoice
func prepareForkchoiceState(
	_ context.Context,
	slot primitives.Slot,
	blockRoot [32]byte,
	parentRoot [32]byte,
	payloadHash [32]byte,
	justifiedEpoch primitives.Epoch,
	finalizedEpoch primitives.Epoch,
) (state.BeaconState, [32]byte, error) {
	blockHeader := &ethpb.BeaconBlockHeader{
		ParentRoot: parentRoot[:],
	}

	executionHeader := &enginev1.ExecutionPayloadHeader{
		BlockHash: payloadHash[:],
	}

	justifiedCheckpoint := &ethpb.Checkpoint{
		Epoch: justifiedEpoch,
	}

	finalizedCheckpoint := &ethpb.Checkpoint{
		Epoch: finalizedEpoch,
	}

	base := &ethpb.BeaconStateBellatrix{
		Slot:                         slot,
		RandaoMixes:                  make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
		CurrentJustifiedCheckpoint:   justifiedCheckpoint,
		FinalizedCheckpoint:          finalizedCheckpoint,
		LatestExecutionPayloadHeader: executionHeader,
		LatestBlockHeader:            blockHeader,
	}

	st, err := state_native.InitializeFromProtoBellatrix(base)
	return st, blockRoot, err
}

func TestForkChoice_UpdateBalancesPositiveChange(t *testing.T) {
	f := setup(0, 0)
	ctx := context.Background()
	st, blkRoot, err := prepareForkchoiceState(ctx, 1, indexToHash(1), params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 2, indexToHash(2), indexToHash(1), params.BeaconConfig().ZeroHash, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 3, indexToHash(3), indexToHash(2), params.BeaconConfig().ZeroHash, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))

	f.votes = []Vote{
		{indexToHash(1), indexToHash(1), 0},
		{indexToHash(2), indexToHash(2), 0},
		{indexToHash(3), indexToHash(3), 0},
	}

	// Each node gets one unique vote. The weight should look like 103 <- 102 <- 101 because
	// they get propagated back.
	f.justifiedBalances = []uint64{10, 20, 30}
	require.NoError(t, f.updateBalances())
	s := f.store
	assert.Equal(t, uint64(10), s.nodeByRoot[indexToHash(1)].balance)
	assert.Equal(t, uint64(20), s.nodeByRoot[indexToHash(2)].balance)
	assert.Equal(t, uint64(30), s.nodeByRoot[indexToHash(3)].balance)
}

func TestForkChoice_UpdateBalancesNegativeChange(t *testing.T) {
	f := setup(0, 0)
	ctx := context.Background()
	st, blkRoot, err := prepareForkchoiceState(ctx, 1, indexToHash(1), params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 2, indexToHash(2), indexToHash(1), params.BeaconConfig().ZeroHash, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 3, indexToHash(3), indexToHash(2), params.BeaconConfig().ZeroHash, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	s := f.store
	s.nodeByRoot[indexToHash(1)].balance = 100
	s.nodeByRoot[indexToHash(2)].balance = 100
	s.nodeByRoot[indexToHash(3)].balance = 100

	f.balances = []uint64{100, 100, 100}
	f.votes = []Vote{
		{indexToHash(1), indexToHash(1), 0},
		{indexToHash(2), indexToHash(2), 0},
		{indexToHash(3), indexToHash(3), 0},
	}

	f.justifiedBalances = []uint64{10, 20, 30}
	require.NoError(t, f.updateBalances())
	assert.Equal(t, uint64(10), s.nodeByRoot[indexToHash(1)].balance)
	assert.Equal(t, uint64(20), s.nodeByRoot[indexToHash(2)].balance)
	assert.Equal(t, uint64(30), s.nodeByRoot[indexToHash(3)].balance)
}

func TestForkChoice_UpdateBalancesUnderflow(t *testing.T) {
	f := setup(0, 0)
	ctx := context.Background()
	st, blkRoot, err := prepareForkchoiceState(ctx, 1, indexToHash(1), params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 2, indexToHash(2), indexToHash(1), params.BeaconConfig().ZeroHash, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 3, indexToHash(3), indexToHash(2), params.BeaconConfig().ZeroHash, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	s := f.store
	s.nodeByRoot[indexToHash(1)].balance = 100
	s.nodeByRoot[indexToHash(2)].balance = 100
	s.nodeByRoot[indexToHash(3)].balance = 100

	f.balances = []uint64{125, 125, 125}
	f.votes = []Vote{
		{indexToHash(1), indexToHash(1), 0},
		{indexToHash(2), indexToHash(2), 0},
		{indexToHash(3), indexToHash(3), 0},
	}

	f.justifiedBalances = []uint64{10, 20, 30}
	require.NoError(t, f.updateBalances())
	assert.Equal(t, uint64(0), s.nodeByRoot[indexToHash(1)].balance)
	assert.Equal(t, uint64(0), s.nodeByRoot[indexToHash(2)].balance)
	assert.Equal(t, uint64(5), s.nodeByRoot[indexToHash(3)].balance)
}

func TestForkChoice_IsCanonical(t *testing.T) {
	f := setup(1, 1)
	ctx := context.Background()
	st, blkRoot, err := prepareForkchoiceState(ctx, 1, indexToHash(1), params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 2, indexToHash(2), params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 3, indexToHash(3), indexToHash(1), params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 4, indexToHash(4), indexToHash(2), params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 5, indexToHash(5), indexToHash(4), params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 6, indexToHash(6), indexToHash(5), params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))

	require.Equal(t, true, f.IsCanonical(params.BeaconConfig().ZeroHash))
	require.Equal(t, false, f.IsCanonical(indexToHash(1)))
	require.Equal(t, true, f.IsCanonical(indexToHash(2)))
	require.Equal(t, false, f.IsCanonical(indexToHash(3)))
	require.Equal(t, true, f.IsCanonical(indexToHash(4)))
	require.Equal(t, true, f.IsCanonical(indexToHash(5)))
	require.Equal(t, true, f.IsCanonical(indexToHash(6)))
}

func TestForkChoice_IsCanonicalReorg(t *testing.T) {
	f := setup(1, 1)
	ctx := context.Background()
	st, blkRoot, err := prepareForkchoiceState(ctx, 1, [32]byte{'1'}, params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 2, [32]byte{'2'}, params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 3, [32]byte{'3'}, [32]byte{'1'}, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 4, [32]byte{'4'}, [32]byte{'2'}, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 5, [32]byte{'5'}, [32]byte{'4'}, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 6, [32]byte{'6'}, [32]byte{'5'}, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))

	f.store.nodeByRoot[[32]byte{'3'}].balance = 10
	require.NoError(t, f.store.treeRootNode.applyWeightChanges(ctx))
	require.Equal(t, uint64(10), f.store.nodeByRoot[[32]byte{'1'}].weight)
	require.Equal(t, uint64(0), f.store.nodeByRoot[[32]byte{'2'}].weight)

	require.NoError(t, f.store.treeRootNode.updateBestDescendant(ctx, 1, 1, 1))
	require.DeepEqual(t, [32]byte{'3'}, f.store.treeRootNode.bestDescendant.root)

	r1 := [32]byte{'1'}
	f.store.justifiedCheckpoint = &forkchoicetypes.Checkpoint{Epoch: 1, Root: r1}
	h, err := f.store.head(ctx)
	require.NoError(t, err)
	require.DeepEqual(t, [32]byte{'3'}, h)
	require.DeepEqual(t, h, f.store.headNode.root)

	require.Equal(t, true, f.IsCanonical(params.BeaconConfig().ZeroHash))
	require.Equal(t, true, f.IsCanonical([32]byte{'1'}))
	require.Equal(t, false, f.IsCanonical([32]byte{'2'}))
	require.Equal(t, true, f.IsCanonical([32]byte{'3'}))
	require.Equal(t, false, f.IsCanonical([32]byte{'4'}))
	require.Equal(t, false, f.IsCanonical([32]byte{'5'}))
	require.Equal(t, false, f.IsCanonical([32]byte{'6'}))
}

func TestForkChoice_AncestorRoot(t *testing.T) {
	f := setup(1, 1)
	ctx := context.Background()
	st, blkRoot, err := prepareForkchoiceState(ctx, 1, indexToHash(1), params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 2, indexToHash(2), indexToHash(1), params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 5, indexToHash(3), indexToHash(2), params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	f.store.treeRootNode = f.store.nodeByRoot[indexToHash(1)]
	f.store.treeRootNode.parent = nil

	r, err := f.AncestorRoot(ctx, indexToHash(3), 6)
	assert.NoError(t, err)
	assert.Equal(t, r, indexToHash(3))

	_, err = f.AncestorRoot(ctx, indexToHash(3), 0)
	assert.ErrorContains(t, ErrNilNode.Error(), err)

	root, err := f.AncestorRoot(ctx, indexToHash(3), 5)
	require.NoError(t, err)
	hash3 := indexToHash(3)
	require.DeepEqual(t, hash3, root)
	root, err = f.AncestorRoot(ctx, indexToHash(3), 1)
	require.NoError(t, err)
	hash1 := indexToHash(1)
	require.DeepEqual(t, hash1, root)
}

func TestForkChoice_AncestorEqualSlot(t *testing.T) {
	f := setup(1, 1)
	ctx := context.Background()
	st, blkRoot, err := prepareForkchoiceState(ctx, 100, [32]byte{'1'}, params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 101, [32]byte{'3'}, [32]byte{'1'}, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))

	r, err := f.AncestorRoot(ctx, [32]byte{'3'}, 100)
	require.NoError(t, err)
	require.Equal(t, r, [32]byte{'1'})
}

func TestForkChoice_AncestorLowerSlot(t *testing.T) {
	f := setup(1, 1)
	ctx := context.Background()
	st, blkRoot, err := prepareForkchoiceState(ctx, 100, [32]byte{'1'}, params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 200, [32]byte{'3'}, [32]byte{'1'}, params.BeaconConfig().ZeroHash, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))

	r, err := f.AncestorRoot(ctx, [32]byte{'3'}, 150)
	require.NoError(t, err)
	require.Equal(t, r, [32]byte{'1'})
}

func TestForkChoice_RemoveEquivocating(t *testing.T) {
	ctx := context.Background()
	f := setup(1, 1)
	// Insert a block it will be head
	st, blkRoot, err := prepareForkchoiceState(ctx, 1, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	head, err := f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'a'}, head)

	// Insert two extra blocks
	st, blkRoot, err = prepareForkchoiceState(ctx, 2, [32]byte{'b'}, [32]byte{'a'}, [32]byte{'B'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 3, [32]byte{'c'}, [32]byte{'a'}, [32]byte{'C'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	head, err = f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'c'}, head)

	// Insert two attestations for block b, one for c it becomes head
	f.ProcessAttestation(ctx, []uint64{1, 2}, [32]byte{'b'}, 1)
	f.ProcessAttestation(ctx, []uint64{3}, [32]byte{'c'}, 1)
	f.justifiedBalances = []uint64{100, 200, 200, 300}
	head, err = f.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'b'}, head)

	// Process b's slashing, c is now head
	f.InsertSlashedIndex(ctx, 1)
	require.Equal(t, uint64(200), f.store.nodeByRoot[[32]byte{'b'}].balance)
	f.justifiedBalances = []uint64{100, 200, 200, 300}
	head, err = f.Head(ctx)
	require.Equal(t, uint64(200), f.store.nodeByRoot[[32]byte{'b'}].weight)
	require.Equal(t, uint64(300), f.store.nodeByRoot[[32]byte{'c'}].weight)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'c'}, head)

	// Process b's slashing again, should be a noop
	f.InsertSlashedIndex(ctx, 1)
	require.Equal(t, uint64(200), f.store.nodeByRoot[[32]byte{'b'}].balance)
	f.justifiedBalances = []uint64{100, 200, 200, 300}
	head, err = f.Head(ctx)
	require.Equal(t, uint64(200), f.store.nodeByRoot[[32]byte{'b'}].weight)
	require.Equal(t, uint64(300), f.store.nodeByRoot[[32]byte{'c'}].weight)
	require.NoError(t, err)
	require.Equal(t, [32]byte{'c'}, head)

	// Process index where index == vote length. Should not panic.
	f.InsertSlashedIndex(ctx, primitives.ValidatorIndex(len(f.balances)))
	f.InsertSlashedIndex(ctx, primitives.ValidatorIndex(len(f.votes)))
	require.Equal(t, true, len(f.store.slashedIndices) > 0)
}

func indexToHash(i uint64) [32]byte {
	var b [8]byte
	binary.LittleEndian.PutUint64(b[:], i)
	return hash.Hash(b[:])
}

func TestForkChoice_UpdateJustifiedAndFinalizedCheckpoints(t *testing.T) {
	f := setup(1, 1)
	ctx := context.Background()
	jr := [32]byte{'j'}
	fr := [32]byte{'f'}
	jc := &forkchoicetypes.Checkpoint{Root: jr, Epoch: 3}
	fc := &forkchoicetypes.Checkpoint{Root: fr, Epoch: 2}
	require.NoError(t, f.UpdateJustifiedCheckpoint(ctx, jc))
	require.NoError(t, f.UpdateFinalizedCheckpoint(fc))
	require.Equal(t, f.store.justifiedCheckpoint.Epoch, jc.Epoch)
	require.Equal(t, f.store.justifiedCheckpoint.Root, jc.Root)
	require.Equal(t, f.store.finalizedCheckpoint.Epoch, fc.Epoch)
	require.Equal(t, f.store.finalizedCheckpoint.Root, fc.Root)
}

func TestStore_CommonAncestor(t *testing.T) {
	ctx := context.Background()
	f := setup(0, 0)

	//  /-- b -- d -- e
	// a
	//  \-- c -- f
	//        \-- g
	//        \ -- h -- i -- j
	st, blkRoot, err := prepareForkchoiceState(ctx, 0, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 1, [32]byte{'b'}, [32]byte{'a'}, [32]byte{'B'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 2, [32]byte{'c'}, [32]byte{'a'}, [32]byte{'C'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 3, [32]byte{'d'}, [32]byte{'b'}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 4, [32]byte{'e'}, [32]byte{'d'}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 5, [32]byte{'f'}, [32]byte{'c'}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 6, [32]byte{'g'}, [32]byte{'c'}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 7, [32]byte{'h'}, [32]byte{'c'}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 8, [32]byte{'i'}, [32]byte{'h'}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 9, [32]byte{'j'}, [32]byte{'i'}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))

	tests := []struct {
		name     string
		r1       [32]byte
		r2       [32]byte
		wantRoot [32]byte
		wantSlot primitives.Slot
	}{
		{
			name:     "Common ancestor between c and b is a",
			r1:       [32]byte{'c'},
			r2:       [32]byte{'b'},
			wantRoot: [32]byte{'a'},
			wantSlot: 0,
		},
		{
			name:     "Common ancestor between c and d is a",
			r1:       [32]byte{'c'},
			r2:       [32]byte{'d'},
			wantRoot: [32]byte{'a'},
			wantSlot: 0,
		},
		{
			name:     "Common ancestor between c and e is a",
			r1:       [32]byte{'c'},
			r2:       [32]byte{'e'},
			wantRoot: [32]byte{'a'},
			wantSlot: 0,
		},
		{
			name:     "Common ancestor between g and f is c",
			r1:       [32]byte{'g'},
			r2:       [32]byte{'f'},
			wantRoot: [32]byte{'c'},
			wantSlot: 2,
		},
		{
			name:     "Common ancestor between f and h is c",
			r1:       [32]byte{'f'},
			r2:       [32]byte{'h'},
			wantRoot: [32]byte{'c'},
			wantSlot: 2,
		},
		{
			name:     "Common ancestor between g and h is c",
			r1:       [32]byte{'g'},
			r2:       [32]byte{'h'},
			wantRoot: [32]byte{'c'},
			wantSlot: 2,
		},
		{
			name:     "Common ancestor between b and h is a",
			r1:       [32]byte{'b'},
			r2:       [32]byte{'h'},
			wantRoot: [32]byte{'a'},
			wantSlot: 0,
		},
		{
			name:     "Common ancestor between e and h is a",
			r1:       [32]byte{'e'},
			r2:       [32]byte{'h'},
			wantRoot: [32]byte{'a'},
			wantSlot: 0,
		},
		{
			name:     "Common ancestor between i and f is c",
			r1:       [32]byte{'i'},
			r2:       [32]byte{'f'},
			wantRoot: [32]byte{'c'},
			wantSlot: 2,
		},
		{
			name:     "Common ancestor between e and h is a",
			r1:       [32]byte{'j'},
			r2:       [32]byte{'g'},
			wantRoot: [32]byte{'c'},
			wantSlot: 2,
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			gotRoot, gotSlot, err := f.CommonAncestor(ctx, tc.r1, tc.r2)
			require.NoError(t, err)
			require.Equal(t, tc.wantRoot, gotRoot)
			require.Equal(t, tc.wantSlot, gotSlot)
		})
	}

	// a -- b -- c -- d
	f = setup(0, 0)
	st, blkRoot, err = prepareForkchoiceState(ctx, 0, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 1, [32]byte{'b'}, [32]byte{'a'}, [32]byte{'B'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 2, [32]byte{'c'}, [32]byte{'b'}, [32]byte{'C'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	st, blkRoot, err = prepareForkchoiceState(ctx, 3, [32]byte{'d'}, [32]byte{'c'}, [32]byte{}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))
	tests = []struct {
		name     string
		r1       [32]byte
		r2       [32]byte
		wantRoot [32]byte
		wantSlot primitives.Slot
	}{
		{
			name:     "Common ancestor between a and b is a",
			r1:       [32]byte{'a'},
			r2:       [32]byte{'b'},
			wantRoot: [32]byte{'a'},
			wantSlot: 0,
		},
		{
			name:     "Common ancestor between b and d is b",
			r1:       [32]byte{'d'},
			r2:       [32]byte{'b'},
			wantRoot: [32]byte{'b'},
			wantSlot: 1,
		},
		{
			name:     "Common ancestor between d and a is a",
			r1:       [32]byte{'d'},
			r2:       [32]byte{'a'},
			wantRoot: [32]byte{'a'},
			wantSlot: 0,
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			gotRoot, gotSlot, err := f.CommonAncestor(ctx, tc.r1, tc.r2)
			require.NoError(t, err)
			require.Equal(t, tc.wantRoot, gotRoot)
			require.Equal(t, tc.wantSlot, gotSlot)
		})
	}

	// Equal inputs should return the same root.
	r, s, err := f.CommonAncestor(ctx, [32]byte{'b'}, [32]byte{'b'})
	require.NoError(t, err)
	require.Equal(t, [32]byte{'b'}, r)
	require.Equal(t, primitives.Slot(1), s)
	// Requesting finalized root (last node) should return the same root.
	r, s, err = f.CommonAncestor(ctx, [32]byte{'a'}, [32]byte{'a'})
	require.NoError(t, err)
	require.Equal(t, [32]byte{'a'}, r)
	require.Equal(t, primitives.Slot(0), s)
	// Requesting unknown root
	_, _, err = f.CommonAncestor(ctx, [32]byte{'a'}, [32]byte{'z'})
	require.ErrorIs(t, err, forkchoice.ErrUnknownCommonAncestor)
	_, _, err = f.CommonAncestor(ctx, [32]byte{'z'}, [32]byte{'a'})
	require.ErrorIs(t, err, forkchoice.ErrUnknownCommonAncestor)
	n := &Node{
		slot:                     100,
		root:                     [32]byte{'y'},
		justifiedEpoch:           1,
		unrealizedJustifiedEpoch: 1,
		finalizedEpoch:           1,
		unrealizedFinalizedEpoch: 1,
		optimistic:               true,
	}

	f.store.nodeByRoot[[32]byte{'y'}] = n
	// broken link
	_, _, err = f.CommonAncestor(ctx, [32]byte{'y'}, [32]byte{'a'})
	require.ErrorIs(t, err, forkchoice.ErrUnknownCommonAncestor)
}

func TestStore_InsertChain(t *testing.T) {
	f := setup(1, 1)
	blks := make([]*forkchoicetypes.BlockAndCheckpoints, 0)
	blk := util.NewBeaconBlock()
	blk.Block.Slot = 1
	var pr [32]byte
	blk.Block.ParentRoot = pr[:]
	root, err := blk.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	blks = append(blks, &forkchoicetypes.BlockAndCheckpoints{Block: wsb.Block(),
		JustifiedCheckpoint: &ethpb.Checkpoint{Epoch: 1, Root: params.BeaconConfig().ZeroHash[:]},
		FinalizedCheckpoint: &ethpb.Checkpoint{Epoch: 1, Root: params.BeaconConfig().ZeroHash[:]},
	})
	for i := uint64(2); i < 11; i++ {
		blk := util.NewBeaconBlock()
		blk.Block.Slot = primitives.Slot(i)
		copiedRoot := root
		blk.Block.ParentRoot = copiedRoot[:]
		wsb, err = blocks.NewSignedBeaconBlock(blk)
		require.NoError(t, err)
		blks = append(blks, &forkchoicetypes.BlockAndCheckpoints{Block: wsb.Block(),
			JustifiedCheckpoint: &ethpb.Checkpoint{Epoch: 1, Root: params.BeaconConfig().ZeroHash[:]},
			FinalizedCheckpoint: &ethpb.Checkpoint{Epoch: 1, Root: params.BeaconConfig().ZeroHash[:]},
		})
		root, err = blk.Block.HashTreeRoot()
		require.NoError(t, err)
	}
	args := make([]*forkchoicetypes.BlockAndCheckpoints, 10)
	for i := 0; i < len(blks); i++ {
		args[i] = blks[10-i-1]
	}
	require.NoError(t, f.InsertChain(context.Background(), args))

	f = setup(1, 1)
	require.NoError(t, f.InsertChain(context.Background(), args[2:]))
}

func TestForkChoice_UpdateCheckpoints(t *testing.T) {
	ctx := context.Background()
	tests := []struct {
		name                string
		justified           *forkchoicetypes.Checkpoint
		finalized           *forkchoicetypes.Checkpoint
		newJustified        *forkchoicetypes.Checkpoint
		newFinalized        *forkchoicetypes.Checkpoint
		wantedJustified     *forkchoicetypes.Checkpoint
		wantedBestJustified *forkchoicetypes.Checkpoint
		wantedFinalized     *forkchoicetypes.Checkpoint
		currentSlot         primitives.Slot
		wantedErr           string
	}{
		{
			name:                "lower than store justified and finalized",
			justified:           &forkchoicetypes.Checkpoint{Epoch: 2, Root: [32]byte{'j'}},
			finalized:           &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'f'}},
			newJustified:        &forkchoicetypes.Checkpoint{Epoch: 1},
			newFinalized:        &forkchoicetypes.Checkpoint{Epoch: 0},
			wantedJustified:     &forkchoicetypes.Checkpoint{Epoch: 2, Root: [32]byte{'j'}},
			wantedBestJustified: &forkchoicetypes.Checkpoint{Epoch: 2, Root: [32]byte{'j'}},
			wantedFinalized:     &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'f'}},
		},
		{
			name:                "higher than store justified, early slot, direct descendant",
			justified:           &forkchoicetypes.Checkpoint{Epoch: 2, Root: [32]byte{'j'}},
			finalized:           &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'f'}},
			newJustified:        &forkchoicetypes.Checkpoint{Epoch: 3, Root: [32]byte{'b'}},
			newFinalized:        &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'g'}},
			wantedJustified:     &forkchoicetypes.Checkpoint{Epoch: 3, Root: [32]byte{'b'}},
			wantedBestJustified: &forkchoicetypes.Checkpoint{Epoch: 3, Root: [32]byte{'b'}},
			wantedFinalized:     &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'f'}},
		},
		{
			name:                "higher than store justified, early slot, not a descendant",
			justified:           &forkchoicetypes.Checkpoint{Epoch: 2, Root: [32]byte{'j'}},
			finalized:           &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'f'}},
			newJustified:        &forkchoicetypes.Checkpoint{Epoch: 3, Root: [32]byte{'c'}},
			newFinalized:        &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'g'}},
			wantedJustified:     &forkchoicetypes.Checkpoint{Epoch: 3, Root: [32]byte{'c'}},
			wantedBestJustified: &forkchoicetypes.Checkpoint{Epoch: 3, Root: [32]byte{'c'}},
			wantedFinalized:     &forkchoicetypes.Checkpoint{Epoch: 1, Root: [32]byte{'f'}},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fcs := setup(tt.justified.Epoch, tt.finalized.Epoch)
			fcs.store.justifiedCheckpoint = tt.justified
			fcs.store.finalizedCheckpoint = tt.finalized
			fcs.store.genesisTime = uint64(time.Now().Unix()) - uint64(tt.currentSlot)*params.BeaconConfig().SecondsPerSlot

			st, blkRoot, err := prepareForkchoiceState(ctx, 32, [32]byte{'f'},
				[32]byte{}, [32]byte{}, tt.finalized.Epoch, tt.finalized.Epoch)
			require.NoError(t, err)
			require.NoError(t, fcs.InsertNode(ctx, st, blkRoot))
			st, blkRoot, err = prepareForkchoiceState(ctx, 64, [32]byte{'j'},
				[32]byte{'f'}, [32]byte{}, tt.justified.Epoch, tt.finalized.Epoch)
			require.NoError(t, err)
			require.NoError(t, fcs.InsertNode(ctx, st, blkRoot))
			st, blkRoot, err = prepareForkchoiceState(ctx, 96, [32]byte{'b'},
				[32]byte{'j'}, [32]byte{}, tt.newJustified.Epoch, tt.newFinalized.Epoch)
			require.NoError(t, err)
			require.NoError(t, fcs.InsertNode(ctx, st, blkRoot))
			st, blkRoot, err = prepareForkchoiceState(ctx, 96, [32]byte{'c'},
				[32]byte{'f'}, [32]byte{}, tt.newJustified.Epoch, tt.newFinalized.Epoch)
			require.NoError(t, err)
			require.NoError(t, fcs.InsertNode(ctx, st, blkRoot))
			st, blkRoot, err = prepareForkchoiceState(ctx, 65, [32]byte{'h'},
				[32]byte{'f'}, [32]byte{}, tt.newFinalized.Epoch, tt.newFinalized.Epoch)
			require.NoError(t, err)
			require.NoError(t, fcs.InsertNode(ctx, st, blkRoot))
			// restart justifications cause insertion messed it up
			fcs.store.justifiedCheckpoint = tt.justified
			fcs.store.finalizedCheckpoint = tt.finalized

			jc := &ethpb.Checkpoint{Epoch: tt.newJustified.Epoch, Root: tt.newJustified.Root[:]}
			fc := &ethpb.Checkpoint{Epoch: tt.newFinalized.Epoch, Root: tt.newFinalized.Root[:]}
			err = fcs.updateCheckpoints(ctx, jc, fc)
			if len(tt.wantedErr) > 0 {
				require.ErrorContains(t, tt.wantedErr, err)
			} else {
				require.NoError(t, err)
				require.Equal(t, tt.wantedJustified.Epoch, fcs.store.justifiedCheckpoint.Epoch)
				require.Equal(t, tt.wantedFinalized.Epoch, fcs.store.finalizedCheckpoint.Epoch)
				require.Equal(t, tt.wantedJustified.Root, fcs.store.justifiedCheckpoint.Root)
				require.Equal(t, tt.wantedFinalized.Root, fcs.store.finalizedCheckpoint.Root)
			}
		})
	}
}

func TestWeight(t *testing.T) {
	ctx := context.Background()
	f := setup(0, 0)

	root := [32]byte{'a'}
	st, blkRoot, err := prepareForkchoiceState(ctx, 0, root, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))

	n, ok := f.store.nodeByRoot[root]
	require.Equal(t, true, ok)
	n.weight = 10
	w, err := f.Weight(root)
	require.NoError(t, err)
	require.Equal(t, uint64(10), w)

	w, err = f.Weight([32]byte{'b'})
	require.ErrorIs(t, err, ErrNilNode)
	require.Equal(t, uint64(0), w)
}

func TestForkchoice_UpdateJustifiedBalances(t *testing.T) {
	f := setup(0, 0)
	balances := []uint64{10, 0, 0, 40, 50, 60, 0, 80, 90, 100}
	f.balancesByRoot = func(context.Context, [32]byte) ([]uint64, error) {
		return balances, nil
	}
	require.NoError(t, f.updateJustifiedBalances(context.Background(), [32]byte{}))
	require.Equal(t, uint64(7), f.numActiveValidators)
	require.Equal(t, uint64(430)/32, f.store.committeeWeight)
	require.DeepEqual(t, balances, f.justifiedBalances)
}

func TestForkChoice_UnrealizedJustifiedPayloadBlockHash(t *testing.T) {
	ctx := context.Background()
	f := setup(0, 0)

	st, blkRoot, err := prepareForkchoiceState(ctx, 0, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 1, 1)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, blkRoot))

	f.store.unrealizedJustifiedCheckpoint.Root = [32]byte{'a'}
	got := f.UnrealizedJustifiedPayloadBlockHash()
	require.Equal(t, [32]byte{'A'}, got)
}

func TestForkChoiceIsViableForCheckpoint(t *testing.T) {
	f := setup(0, 0)
	ctx := context.Background()

	st, root, err := prepareForkchoiceState(ctx, 0, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 0, 0)
	require.NoError(t, err)
	// No Node
	viable, err := f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: root})
	require.NoError(t, err)
	require.Equal(t, false, viable)

	// No Children
	require.NoError(t, f.InsertNode(ctx, st, root))
	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: root, Epoch: 0})
	require.NoError(t, err)
	require.Equal(t, true, viable)

	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: root, Epoch: 1})
	require.NoError(t, err)
	require.Equal(t, true, viable)

	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: root, Epoch: 2})
	require.NoError(t, err)
	require.Equal(t, true, viable)

	st, bRoot, err := prepareForkchoiceState(ctx, 1, [32]byte{'b'}, root, [32]byte{'B'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, bRoot))

	// Epoch start
	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: root})
	require.NoError(t, err)
	require.Equal(t, true, viable)

	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: root, Epoch: 1})
	require.NoError(t, err)
	require.Equal(t, false, viable)

	// No Children but impossible checkpoint
	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: bRoot})
	require.NoError(t, err)
	require.Equal(t, false, viable)

	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: bRoot, Epoch: 1})
	require.NoError(t, err)
	require.Equal(t, true, viable)

	st, cRoot, err := prepareForkchoiceState(ctx, 2, [32]byte{'c'}, bRoot, [32]byte{'C'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, cRoot))

	// Children in same epoch
	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: bRoot})
	require.NoError(t, err)
	require.Equal(t, false, viable)

	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: bRoot, Epoch: 1})
	require.NoError(t, err)
	require.Equal(t, false, viable)

	st, dRoot, err := prepareForkchoiceState(ctx, params.BeaconConfig().SlotsPerEpoch, [32]byte{'d'}, bRoot, [32]byte{'D'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, dRoot))

	// Children in next epoch but boundary
	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: bRoot})
	require.NoError(t, err)
	require.Equal(t, false, viable)

	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: bRoot, Epoch: 1})
	require.NoError(t, err)
	require.Equal(t, false, viable)

	// Boundary block
	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: dRoot, Epoch: 1})
	require.NoError(t, err)
	require.Equal(t, true, viable)

	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: dRoot, Epoch: 0})
	require.NoError(t, err)
	require.Equal(t, false, viable)

	// Children in next epoch
	st, eRoot, err := prepareForkchoiceState(ctx, params.BeaconConfig().SlotsPerEpoch+1, [32]byte{'e'}, bRoot, [32]byte{'E'}, 0, 0)
	require.NoError(t, err)
	require.NoError(t, f.InsertNode(ctx, st, eRoot))

	viable, err = f.IsViableForCheckpoint(&forkchoicetypes.Checkpoint{Root: bRoot, Epoch: 1})
	require.NoError(t, err)
	require.Equal(t, true, viable)
}

func TestForkChoiceSlot(t *testing.T) {
	f := setup(0, 0)
	ctx := context.Background()
	st, root, err := prepareForkchoiceState(ctx, 3, [32]byte{'a'}, params.BeaconConfig().ZeroHash, [32]byte{'A'}, 0, 0)
	require.NoError(t, err)
	// No Node
	_, err = f.Slot(root)
	require.ErrorIs(t, ErrNilNode, err)

	require.NoError(t, f.InsertNode(ctx, st, root))
	slot, err := f.Slot(root)
	require.NoError(t, err)
	require.Equal(t, primitives.Slot(3), slot)
}
