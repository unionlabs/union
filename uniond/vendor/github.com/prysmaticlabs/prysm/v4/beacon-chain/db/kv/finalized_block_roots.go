package kv

import (
	"bytes"
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filters"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
)

var previousFinalizedCheckpointKey = []byte("previous-finalized-checkpoint")

// Blocks from the recent finalized epoch are not part of the finalized and canonical chain in this
// index. These containers will be removed on the next update of finalized checkpoint. Note that
// these block roots may be considered canonical in the "head view" of the beacon chain, but not so
// in this index.
var containerFinalizedButNotCanonical = []byte("recent block needs reindexing to determine canonical")

// The finalized block roots index tracks beacon blocks which are finalized in the canonical chain.
// The finalized checkpoint contains the epoch which was finalized and the highest beacon block
// root where block.slot <= start_slot(epoch). As a result, we cannot index the finalized canonical
// beacon block chain using the finalized root alone as this would exclude all other blocks in the
// finalized epoch from being indexed as "final and canonical".
//
// The algorithm for building the index works as follows:
//   - De-index all finalized beacon block roots from previous_finalized_epoch to
//     new_finalized_epoch. (I.e. delete these roots from the index, to be re-indexed.)
//   - Build the canonical finalized chain by walking up the ancestry chain from the finalized block
//     root until a parent is found in the index, or the parent is genesis or the origin checkpoint.
//   - Add all block roots in the database where epoch(block.slot) == checkpoint.epoch.
//
// This method ensures that all blocks from the current finalized epoch are considered "final" while
// maintaining only canonical and finalized blocks older than the current finalized epoch.
func (s *Store) updateFinalizedBlockRoots(ctx context.Context, tx *bolt.Tx, checkpoint *ethpb.Checkpoint) error {
	ctx, span := trace.StartSpan(ctx, "BeaconDB.updateFinalizedBlockRoots")
	defer span.End()

	bkt := tx.Bucket(finalizedBlockRootsIndexBucket)

	root := checkpoint.Root
	var previousRoot []byte
	genesisRoot := tx.Bucket(blocksBucket).Get(genesisBlockRootKey)
	initCheckpointRoot := tx.Bucket(blocksBucket).Get(originCheckpointBlockRootKey)

	// De-index recent finalized block roots, to be re-indexed.
	previousFinalizedCheckpoint := &ethpb.Checkpoint{}
	if b := bkt.Get(previousFinalizedCheckpointKey); b != nil {
		if err := decode(ctx, b, previousFinalizedCheckpoint); err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
	}

	blockRoots, err := s.BlockRoots(ctx, filters.NewFilter().
		SetStartEpoch(previousFinalizedCheckpoint.Epoch).
		SetEndEpoch(checkpoint.Epoch+1),
	)
	if err != nil {
		tracing.AnnotateError(span, err)
		return err
	}
	for _, root := range blockRoots {
		if err := bkt.Delete(root[:]); err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
	}

	// Walk up the ancestry chain until we reach a block root present in the finalized block roots
	// index bucket or genesis block root.
	for {
		if bytes.Equal(root, genesisRoot) {
			break
		}

		signedBlock, err := s.Block(ctx, bytesutil.ToBytes32(root))
		if err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
		if err := blocks.BeaconBlockIsNil(signedBlock); err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
		block := signedBlock.Block()

		parentRoot := block.ParentRoot()
		container := &ethpb.FinalizedBlockRootContainer{
			ParentRoot: parentRoot[:],
			ChildRoot:  previousRoot,
		}

		enc, err := encode(ctx, container)
		if err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
		if err := bkt.Put(root, enc); err != nil {
			tracing.AnnotateError(span, err)
			return err
		}

		// breaking here allows the initial checkpoint root to be correctly inserted,
		// but stops the loop from trying to search for its parent.
		if bytes.Equal(root, initCheckpointRoot) {
			break
		}

		// Found parent, loop exit condition.
		pr := block.ParentRoot()
		if parentBytes := bkt.Get(pr[:]); parentBytes != nil {
			parent := &ethpb.FinalizedBlockRootContainer{}
			if err := decode(ctx, parentBytes, parent); err != nil {
				tracing.AnnotateError(span, err)
				return err
			}
			parent.ChildRoot = root
			enc, err := encode(ctx, parent)
			if err != nil {
				tracing.AnnotateError(span, err)
				return err
			}
			if err := bkt.Put(pr[:], enc); err != nil {
				tracing.AnnotateError(span, err)
				return err
			}
			break
		}
		previousRoot = root
		root = pr[:]
	}

	// Upsert blocks from the current finalized epoch.
	roots, err := s.BlockRoots(ctx, filters.NewFilter().SetStartEpoch(checkpoint.Epoch).SetEndEpoch(checkpoint.Epoch+1))
	if err != nil {
		tracing.AnnotateError(span, err)
		return err
	}
	for _, root := range roots {
		root := root[:]
		if bytes.Equal(root, checkpoint.Root) || bkt.Get(root) != nil {
			continue
		}
		if err := bkt.Put(root, containerFinalizedButNotCanonical); err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
	}

	// Update previous checkpoint
	enc, err := encode(ctx, checkpoint)
	if err != nil {
		tracing.AnnotateError(span, err)
		return err
	}

	return bkt.Put(previousFinalizedCheckpointKey, enc)
}

// BackfillFinalizedIndex updates the finalized index for a contiguous chain of blocks that are the ancestors of the
// given finalized child root. This is needed to update the finalized index during backfill, because the usual
// updateFinalizedBlockRoots has assumptions that are incompatible with backfill processing.
func (s *Store) BackfillFinalizedIndex(ctx context.Context, blocks []blocks.ROBlock, finalizedChildRoot [32]byte) error {
	ctx, span := trace.StartSpan(ctx, "BeaconDB.BackfillFinalizedIndex")
	defer span.End()
	if len(blocks) == 0 {
		return errEmptyBlockSlice
	}

	fbrs := make([]*ethpb.FinalizedBlockRootContainer, len(blocks))
	encs := make([][]byte, len(blocks))
	for i := range blocks {
		pr := blocks[i].Block().ParentRoot()
		fbrs[i] = &ethpb.FinalizedBlockRootContainer{
			ParentRoot: pr[:],
			// ChildRoot: will be filled in on the next iteration when we look at the descendent block.
		}
		if i == 0 {
			continue
		}
		if blocks[i-1].Root() != blocks[i].Block().ParentRoot() {
			return errors.Wrapf(errIncorrectBlockParent, "previous root=%#x, slot=%d; child parent_root=%#x, root=%#x, slot=%d",
				blocks[i-1].Root(), blocks[i-1].Block().Slot(), blocks[i].Block().ParentRoot(), blocks[i].Root(), blocks[i].Block().Slot())
		}

		// We know the previous index is the parent of this one thanks to the assertion above,
		// so we can set the ChildRoot of the previous value to the root of the current value.
		fbrs[i-1].ChildRoot = blocks[i].RootSlice()
		// Now that the value for fbrs[i-1] is complete, perform encoding here to minimize time in Update,
		// which holds the global db lock.
		penc, err := encode(ctx, fbrs[i-1])
		if err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
		encs[i-1] = penc

		// The final element is the parent of finalizedChildRoot. This is checked inside the db transaction using
		// the parent_root value stored in the index data for finalizedChildRoot.
		if i == len(blocks)-1 {
			fbrs[i].ChildRoot = finalizedChildRoot[:]
			// Final element is complete, so it is pre-encoded like the others.
			enc, err := encode(ctx, fbrs[i])
			if err != nil {
				tracing.AnnotateError(span, err)
				return err
			}
			encs[i] = enc
		}
	}

	return s.db.Update(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(finalizedBlockRootsIndexBucket)
		child := bkt.Get(finalizedChildRoot[:])
		if len(child) == 0 {
			return errFinalizedChildNotFound
		}
		fcc := &ethpb.FinalizedBlockRootContainer{}
		if err := decode(ctx, child, fcc); err != nil {
			return errors.Wrapf(err, "unable to decode finalized block root container for root=%#x", finalizedChildRoot)
		}
		// Ensure that the existing finalized chain descends from the new segment.
		if !bytes.Equal(fcc.ParentRoot, blocks[len(blocks)-1].RootSlice()) {
			return errors.Wrapf(errNotConnectedToFinalized, "finalized block root container for root=%#x has parent_root=%#x, not %#x",
				finalizedChildRoot, fcc.ParentRoot, blocks[len(blocks)-1].RootSlice())
		}
		// Update the finalized index with entries for each block in the new segment.
		for i := range fbrs {
			if err := bkt.Put(blocks[i].RootSlice(), encs[i]); err != nil {
				return err
			}
		}
		return nil
	})
}

// IsFinalizedBlock returns true if the block root is present in the finalized block root index.
// A beacon block root contained exists in this index if it is considered finalized and canonical.
// Note: beacon blocks from the latest finalized epoch return true, whether or not they are
// considered canonical in the "head view" of the beacon node.
func (s *Store) IsFinalizedBlock(ctx context.Context, blockRoot [32]byte) bool {
	_, span := trace.StartSpan(ctx, "BeaconDB.IsFinalizedBlock")
	defer span.End()

	var exists bool
	err := s.db.View(func(tx *bolt.Tx) error {
		exists = tx.Bucket(finalizedBlockRootsIndexBucket).Get(blockRoot[:]) != nil
		// Check genesis block root.
		if !exists {
			genRoot := tx.Bucket(blocksBucket).Get(genesisBlockRootKey)
			exists = bytesutil.ToBytes32(genRoot) == blockRoot
		}
		return nil
	})
	if err != nil {
		tracing.AnnotateError(span, err)
	}
	return exists
}

// FinalizedChildBlock returns the child block of a provided finalized block. If
// no finalized block or its respective child block exists we return with a nil
// block.
func (s *Store) FinalizedChildBlock(ctx context.Context, blockRoot [32]byte) (interfaces.ReadOnlySignedBeaconBlock, error) {
	ctx, span := trace.StartSpan(ctx, "BeaconDB.FinalizedChildBlock")
	defer span.End()

	var blk interfaces.ReadOnlySignedBeaconBlock
	err := s.db.View(func(tx *bolt.Tx) error {
		blkBytes := tx.Bucket(finalizedBlockRootsIndexBucket).Get(blockRoot[:])
		if blkBytes == nil {
			return nil
		}
		if bytes.Equal(blkBytes, containerFinalizedButNotCanonical) {
			return nil
		}
		ctr := &ethpb.FinalizedBlockRootContainer{}
		if err := decode(ctx, blkBytes, ctr); err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
		enc := tx.Bucket(blocksBucket).Get(ctr.ChildRoot)
		if enc == nil {
			return nil
		}
		var err error
		blk, err = unmarshalBlock(ctx, enc)
		return err
	})
	tracing.AnnotateError(span, err)
	return blk, err
}
