package sync

import (
	"math/rand"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestSortedObj_SortBlocksRoots(t *testing.T) {
	source := rand.NewSource(33)
	randGen := rand.New(source)
	randFunc := func() int64 {
		return randGen.Int63n(50)
	}

	var blks []blocks.ROBlock
	for i := 0; i < 10; i++ {
		slot := primitives.Slot(randFunc())
		newBlk, err := blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlock{Block: &ethpb.BeaconBlock{Slot: slot, Body: &ethpb.BeaconBlockBody{}}})
		require.NoError(t, err)
		root := bytesutil.ToBytes32(bytesutil.Bytes32(uint64(slot)))
		b, err := blocks.NewROBlockWithRoot(newBlk, root)
		require.NoError(t, err)
		blks = append(blks, b)
	}

	newBlks := sortedUniqueBlocks(blks)
	previousSlot := primitives.Slot(0)
	for _, b := range newBlks {
		if b.Block().Slot() < previousSlot {
			t.Errorf("Block list is not sorted as %d is smaller than previousSlot %d", b.Block().Slot(), previousSlot)
		}
		previousSlot = b.Block().Slot()
	}
}

func TestSortedObj_NoDuplicates(t *testing.T) {
	source := rand.NewSource(33)
	randGen := rand.New(source)
	var blks []blocks.ROBlock
	randFunc := func() int64 {
		return randGen.Int63n(50)
	}

	for i := 0; i < 10; i++ {
		slot := primitives.Slot(randFunc())
		newBlk := &ethpb.SignedBeaconBlock{Block: &ethpb.BeaconBlock{Slot: slot, Body: &ethpb.BeaconBlockBody{}}}
		// append twice
		wsb, err := blocks.NewSignedBeaconBlock(newBlk)
		require.NoError(t, err)
		wsbCopy, err := wsb.Copy()
		require.NoError(t, err)
		root := bytesutil.ToBytes32(bytesutil.Bytes32(uint64(slot)))
		b, err := blocks.NewROBlockWithRoot(wsb, root)
		require.NoError(t, err)
		b2, err := blocks.NewROBlockWithRoot(wsbCopy, root)
		require.NoError(t, err)
		blks = append(blks, b, b2)
	}

	dedup := sortedUniqueBlocks(blks)
	roots := make(map[[32]byte]int)
	for i, b := range dedup {
		if di, dup := roots[b.Root()]; dup {
			t.Errorf("Duplicated root %#x at index %d and %d", b.Root(), di, i)
		}
		roots[b.Root()] = i
	}
}

func TestBlockBatchNext(t *testing.T) {
	cases := []struct {
		name   string
		batch  blockBatch
		start  primitives.Slot
		reqEnd primitives.Slot
		size   uint64
		next   []blockBatch
		more   []bool
		err    error
	}{
		{
			name:   "end aligned",
			batch:  blockBatch{start: 0, end: 20},
			start:  0,
			reqEnd: 40,
			size:   20,
			next: []blockBatch{
				{start: 0, end: 19},
				{start: 20, end: 39},
				{start: 40, end: 40},
				{},
			},
			more: []bool{true, true, true, false},
		},
		{
			name:   "batches with more",
			batch:  blockBatch{start: 0, end: 22},
			start:  0,
			reqEnd: 40,
			size:   23,
			next: []blockBatch{
				{start: 0, end: 22},
				{start: 23, end: 40},
				{},
			},
			more: []bool{true, true, false},
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			var next blockBatch
			var more bool
			i := 0
			for next, more = newBlockBatch(c.start, c.reqEnd, c.size); more; next, more = next.next(c.reqEnd, c.size) {
				exp := c.next[i]
				require.Equal(t, c.more[i], more)
				require.Equal(t, exp.start, next.start)
				require.Equal(t, exp.end, next.end)
				if exp.err != nil {
					require.ErrorIs(t, next.err, exp.err)
				} else {
					require.NoError(t, next.err)
				}
				i++
			}
		})
	}
}

func TestZeroSizeNoOp(t *testing.T) {
	_, more := newBlockBatch(12345, 12345, 0)
	require.Equal(t, false, more)
}
