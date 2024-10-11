package kv

import (
	"bytes"
	"context"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	bolt "go.etcd.io/bbolt"
)

var genesisBlockRoot = bytesutil.ToBytes32([]byte{'G', 'E', 'N', 'E', 'S', 'I', 'S'})

func TestStore_IsFinalizedBlock(t *testing.T) {
	slotsPerEpoch := uint64(params.BeaconConfig().SlotsPerEpoch)
	db := setupDB(t)
	ctx := context.Background()

	require.NoError(t, db.SaveGenesisBlockRoot(ctx, genesisBlockRoot))

	blks := makeBlocks(t, 0, slotsPerEpoch*3, genesisBlockRoot)
	require.NoError(t, db.SaveBlocks(ctx, blks))

	root, err := blks[slotsPerEpoch].Block().HashTreeRoot()
	require.NoError(t, err)

	cp := &ethpb.Checkpoint{
		Epoch: 1,
		Root:  root[:],
	}

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	// a state is required to save checkpoint
	require.NoError(t, db.SaveState(ctx, st, root))
	require.NoError(t, db.SaveFinalizedCheckpoint(ctx, cp))

	// All blocks up to slotsPerEpoch*2 should be in the finalized index.
	for i := uint64(0); i < slotsPerEpoch*2; i++ {
		root, err := blks[i].Block().HashTreeRoot()
		require.NoError(t, err)
		assert.Equal(t, true, db.IsFinalizedBlock(ctx, root), "Block at index %d was not considered finalized in the index", i)
	}
	for i := slotsPerEpoch * 3; i < uint64(len(blks)); i++ {
		root, err := blks[i].Block().HashTreeRoot()
		require.NoError(t, err)
		assert.Equal(t, false, db.IsFinalizedBlock(ctx, root), "Block at index %d was considered finalized in the index, but should not have", i)
	}
}

func TestStore_IsFinalizedBlockGenesis(t *testing.T) {
	db := setupDB(t)
	ctx := context.Background()

	blk := util.NewBeaconBlock()
	blk.Block.Slot = 0
	root, err := blk.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err := consensusblocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	require.NoError(t, db.SaveBlock(ctx, wsb))
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))
	assert.Equal(t, true, db.IsFinalizedBlock(ctx, root), "Finalized genesis block doesn't exist in db")
}

// This test scenario is to test a specific edge case where the finalized block root is not part of
// the finalized and canonical chain.
//
// Example:
// 0    1  2  3   4     5   6     slot
// a <- b <-- d <- e <- f <- g    roots
//
//	^- c
//
// Imagine that epochs are 2 slots and that epoch 1, 2, and 3 are finalized. Checkpoint roots would
// be c, e, and g. In this scenario, c was a finalized checkpoint root but no block built upon it so
// it should not be considered "final and canonical" in the view at slot 6.
func TestStore_IsFinalized_ForkEdgeCase(t *testing.T) {
	slotsPerEpoch := uint64(params.BeaconConfig().SlotsPerEpoch)
	blocks0 := makeBlocks(t, slotsPerEpoch*0, slotsPerEpoch, genesisBlockRoot)
	blocks1 := append(
		makeBlocks(t, slotsPerEpoch*1, 1, bytesutil.ToBytes32(sszRootOrDie(t, blocks0[len(blocks0)-1]))), // No block builds off of the first block in epoch.
		makeBlocks(t, slotsPerEpoch*1+1, slotsPerEpoch-1, bytesutil.ToBytes32(sszRootOrDie(t, blocks0[len(blocks0)-1])))...,
	)
	blocks2 := makeBlocks(t, slotsPerEpoch*2, slotsPerEpoch, bytesutil.ToBytes32(sszRootOrDie(t, blocks1[len(blocks1)-1])))

	db := setupDB(t)
	ctx := context.Background()

	require.NoError(t, db.SaveGenesisBlockRoot(ctx, genesisBlockRoot))
	require.NoError(t, db.SaveBlocks(ctx, blocks0))
	require.NoError(t, db.SaveBlocks(ctx, blocks1))
	require.NoError(t, db.SaveBlocks(ctx, blocks2))

	// First checkpoint
	checkpoint1 := &ethpb.Checkpoint{
		Root:  sszRootOrDie(t, blocks1[0]),
		Epoch: 1,
	}

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	// A state is required to save checkpoint
	require.NoError(t, db.SaveState(ctx, st, bytesutil.ToBytes32(checkpoint1.Root)))
	require.NoError(t, db.SaveFinalizedCheckpoint(ctx, checkpoint1))
	// All blocks in blocks0 and blocks1 should be finalized and canonical.
	for i, block := range append(blocks0, blocks1...) {
		root := sszRootOrDie(t, block)
		assert.Equal(t, true, db.IsFinalizedBlock(ctx, bytesutil.ToBytes32(root)), "%d - Expected block %#x to be finalized", i, root)
	}

	// Second checkpoint
	checkpoint2 := &ethpb.Checkpoint{
		Root:  sszRootOrDie(t, blocks2[0]),
		Epoch: 2,
	}
	// A state is required to save checkpoint
	require.NoError(t, db.SaveState(ctx, st, bytesutil.ToBytes32(checkpoint2.Root)))
	require.NoError(t, db.SaveFinalizedCheckpoint(ctx, checkpoint2))
	// All blocks in blocks0 and blocks2 should be finalized and canonical.
	for i, block := range append(blocks0, blocks2...) {
		root := sszRootOrDie(t, block)
		assert.Equal(t, true, db.IsFinalizedBlock(ctx, bytesutil.ToBytes32(root)), "%d - Expected block %#x to be finalized", i, root)
	}
	// All blocks in blocks1 should be finalized and canonical, except blocks1[0].
	for i, block := range blocks1 {
		root := sszRootOrDie(t, block)
		if db.IsFinalizedBlock(ctx, bytesutil.ToBytes32(root)) == (i == 0) {
			t.Errorf("Expected db.IsFinalizedBlock(ctx, blocks1[%d]) to be %v", i, i != 0)
		}
	}
}

func TestStore_IsFinalizedChildBlock(t *testing.T) {
	slotsPerEpoch := uint64(params.BeaconConfig().SlotsPerEpoch)
	ctx := context.Background()

	eval := func(t testing.TB, ctx context.Context, db *Store, blks []interfaces.ReadOnlySignedBeaconBlock) {
		require.NoError(t, db.SaveBlocks(ctx, blks))
		root, err := blks[slotsPerEpoch].Block().HashTreeRoot()
		require.NoError(t, err)

		cp := &ethpb.Checkpoint{
			Epoch: 1,
			Root:  root[:],
		}

		st, err := util.NewBeaconState()
		require.NoError(t, err)
		// a state is required to save checkpoint
		require.NoError(t, db.SaveState(ctx, st, root))
		require.NoError(t, db.SaveFinalizedCheckpoint(ctx, cp))

		// All blocks up to slotsPerEpoch should have a finalized child block.
		for i := uint64(0); i < slotsPerEpoch; i++ {
			root, err := blks[i].Block().HashTreeRoot()
			require.NoError(t, err)
			assert.Equal(t, true, db.IsFinalizedBlock(ctx, root), "Block at index %d was not considered finalized in the index", i)
			blk, err := db.FinalizedChildBlock(ctx, root)
			assert.NoError(t, err)
			if blk == nil {
				t.Error("Child block doesn't exist for valid finalized block.")
			}
		}
	}

	setup := func(t testing.TB) *Store {
		db := setupDB(t)
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, genesisBlockRoot))

		return db
	}

	t.Run("phase0", func(t *testing.T) {
		db := setup(t)

		blks := makeBlocks(t, 0, slotsPerEpoch*3, genesisBlockRoot)
		eval(t, ctx, db, blks)
	})

	t.Run("altair", func(t *testing.T) {
		db := setup(t)

		blks := makeBlocksAltair(t, 0, slotsPerEpoch*3, genesisBlockRoot)
		eval(t, ctx, db, blks)
	})
}

func sszRootOrDie(t *testing.T, block interfaces.ReadOnlySignedBeaconBlock) []byte {
	root, err := block.Block().HashTreeRoot()
	require.NoError(t, err)
	return root[:]
}

func makeBlocks(t *testing.T, i, n uint64, previousRoot [32]byte) []interfaces.ReadOnlySignedBeaconBlock {
	blocks := make([]*ethpb.SignedBeaconBlock, n)
	ifaceBlocks := make([]interfaces.ReadOnlySignedBeaconBlock, n)
	for j := i; j < n+i; j++ {
		parentRoot := make([]byte, fieldparams.RootLength)
		copy(parentRoot, previousRoot[:])
		blocks[j-i] = util.NewBeaconBlock()
		blocks[j-i].Block.Slot = primitives.Slot(j + 1)
		blocks[j-i].Block.ParentRoot = parentRoot
		var err error
		previousRoot, err = blocks[j-i].Block.HashTreeRoot()
		require.NoError(t, err)
		ifaceBlocks[j-i], err = consensusblocks.NewSignedBeaconBlock(blocks[j-i])
		require.NoError(t, err)
	}
	return ifaceBlocks
}

func makeBlocksAltair(t *testing.T, startIdx, num uint64, previousRoot [32]byte) []interfaces.ReadOnlySignedBeaconBlock {
	blocks := make([]*ethpb.SignedBeaconBlockAltair, num)
	ifaceBlocks := make([]interfaces.ReadOnlySignedBeaconBlock, num)
	for j := startIdx; j < num+startIdx; j++ {
		parentRoot := make([]byte, fieldparams.RootLength)
		copy(parentRoot, previousRoot[:])
		blocks[j-startIdx] = util.NewBeaconBlockAltair()
		blocks[j-startIdx].Block.Slot = primitives.Slot(j + 1)
		blocks[j-startIdx].Block.ParentRoot = parentRoot
		var err error
		previousRoot, err = blocks[j-startIdx].Block.HashTreeRoot()
		require.NoError(t, err)
		ifaceBlocks[j-startIdx], err = consensusblocks.NewSignedBeaconBlock(blocks[j-startIdx])
		require.NoError(t, err)
	}
	return ifaceBlocks
}

func TestStore_BackfillFinalizedIndex(t *testing.T) {
	db := setupDB(t)
	ctx := context.Background()
	require.ErrorIs(t, db.BackfillFinalizedIndex(ctx, []consensusblocks.ROBlock{}, [32]byte{}), errEmptyBlockSlice)
	blks, err := consensusblocks.NewROBlockSlice(makeBlocks(t, 0, 66, [32]byte{}))
	require.NoError(t, err)

	// set up existing finalized block
	ebpr := blks[64].Block().ParentRoot()
	ebr := blks[64].Root()
	chldr := blks[65].Root()
	ebf := &ethpb.FinalizedBlockRootContainer{
		ParentRoot: ebpr[:],
		ChildRoot:  chldr[:],
	}
	disjoint := []consensusblocks.ROBlock{
		blks[0],
		blks[2],
	}
	enc, err := encode(ctx, ebf)
	require.NoError(t, err)
	err = db.db.Update(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(finalizedBlockRootsIndexBucket)
		return bkt.Put(ebr[:], enc)
	})

	// reslice to remove the existing blocks
	blks = blks[0:64]
	// check the other error conditions with a descendent root that really doesn't exist
	require.NoError(t, err)
	require.ErrorIs(t, db.BackfillFinalizedIndex(ctx, disjoint, [32]byte{}), errIncorrectBlockParent)
	require.NoError(t, err)
	require.ErrorIs(t, errFinalizedChildNotFound, db.BackfillFinalizedIndex(ctx, blks, [32]byte{}))

	// use the real root so that it succeeds
	require.NoError(t, db.BackfillFinalizedIndex(ctx, blks, ebr))
	for i := range blks {
		require.NoError(t, db.db.View(func(tx *bolt.Tx) error {
			bkt := tx.Bucket(finalizedBlockRootsIndexBucket)
			encfr := bkt.Get(blks[i].RootSlice())
			require.Equal(t, true, len(encfr) > 0)
			fr := &ethpb.FinalizedBlockRootContainer{}
			require.NoError(t, decode(ctx, encfr, fr))
			require.Equal(t, 32, len(fr.ParentRoot))
			require.Equal(t, 32, len(fr.ChildRoot))
			pr := blks[i].Block().ParentRoot()
			require.Equal(t, true, bytes.Equal(fr.ParentRoot, pr[:]))
			if i > 0 {
				require.Equal(t, true, bytes.Equal(fr.ParentRoot, blks[i-1].RootSlice()))
			}
			if i < len(blks)-1 {
				require.DeepEqual(t, fr.ChildRoot, blks[i+1].RootSlice())
			}
			if i == len(blks)-1 {
				require.DeepEqual(t, fr.ChildRoot, ebr[:])
			}
			return nil
		}))
	}
}
