package kv

import (
	"context"
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
)

var errMissingStateForCheckpoint = errors.New("missing state summary for checkpoint root")

// JustifiedCheckpoint returns the latest justified checkpoint in beacon chain.
func (s *Store) JustifiedCheckpoint(ctx context.Context) (*ethpb.Checkpoint, error) {
	ctx, span := trace.StartSpan(ctx, "BeaconDB.JustifiedCheckpoint")
	defer span.End()
	var checkpoint *ethpb.Checkpoint
	err := s.db.View(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(checkpointBucket)
		enc := bkt.Get(justifiedCheckpointKey)
		if enc == nil {
			checkpoint = &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
			return nil
		}
		checkpoint = &ethpb.Checkpoint{}
		return decode(ctx, enc, checkpoint)
	})
	return checkpoint, err
}

// FinalizedCheckpoint returns the latest finalized checkpoint in beacon chain.
func (s *Store) FinalizedCheckpoint(ctx context.Context) (*ethpb.Checkpoint, error) {
	ctx, span := trace.StartSpan(ctx, "BeaconDB.FinalizedCheckpoint")
	defer span.End()
	var checkpoint *ethpb.Checkpoint
	err := s.db.View(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(checkpointBucket)
		enc := bkt.Get(finalizedCheckpointKey)
		if enc == nil {
			checkpoint = &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
			return nil
		}
		checkpoint = &ethpb.Checkpoint{}
		return decode(ctx, enc, checkpoint)
	})
	return checkpoint, err
}

// SaveJustifiedCheckpoint saves justified checkpoint in beacon chain.
func (s *Store) SaveJustifiedCheckpoint(ctx context.Context, checkpoint *ethpb.Checkpoint) error {
	ctx, span := trace.StartSpan(ctx, "BeaconDB.SaveJustifiedCheckpoint")
	defer span.End()

	return s.saveCheckpoint(ctx, justifiedCheckpointKey, checkpoint)
}

// SaveFinalizedCheckpoint saves finalized checkpoint in beacon chain.
func (s *Store) SaveFinalizedCheckpoint(ctx context.Context, checkpoint *ethpb.Checkpoint) error {
	ctx, span := trace.StartSpan(ctx, "BeaconDB.SaveFinalizedCheckpoint")
	defer span.End()

	enc, err := encode(ctx, checkpoint)
	if err != nil {
		tracing.AnnotateError(span, err)
		return err
	}
	hasStateSummary := s.HasStateSummary(ctx, bytesutil.ToBytes32(checkpoint.Root))
	err = s.db.Update(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(checkpointBucket)
		hasStateInDB := tx.Bucket(stateBucket).Get(checkpoint.Root) != nil
		if !(hasStateInDB || hasStateSummary) {
			log.Warnf("Recovering state summary for finalized root: %#x", bytesutil.Trunc(checkpoint.Root))
			if err := recoverStateSummary(ctx, tx, checkpoint.Root); err != nil {
				return errors.Wrapf(errMissingStateForCheckpoint, "could not save finalized checkpoint, finalized root: %#x", bytesutil.Trunc(checkpoint.Root))
			}
		}
		if err := bucket.Put(finalizedCheckpointKey, enc); err != nil {
			return err
		}

		return s.updateFinalizedBlockRoots(ctx, tx, checkpoint)
	})
	tracing.AnnotateError(span, err)
	return err
}

func (s *Store) saveCheckpoint(ctx context.Context, key []byte, checkpoint *ethpb.Checkpoint) error {
	ctx, span := trace.StartSpan(ctx, "BeaconDB.saveCheckpoint")
	defer span.End()

	enc, err := encode(ctx, checkpoint)
	if err != nil {
		tracing.AnnotateError(span, err)
		return err
	}
	hasStateSummary := s.HasStateSummary(ctx, bytesutil.ToBytes32(checkpoint.Root))
	err = s.db.Update(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(checkpointBucket)
		hasStateInDB := tx.Bucket(stateBucket).Get(checkpoint.Root) != nil
		if !(hasStateInDB || hasStateSummary) {
			log.WithField("root", fmt.Sprintf("%#x", bytesutil.Trunc(checkpoint.Root))).Warn("Recovering state summary")
			if err := recoverStateSummary(ctx, tx, checkpoint.Root); err != nil {
				return errMissingStateForCheckpoint
			}
		}
		return bucket.Put(key, enc)
	})
	tracing.AnnotateError(span, err)
	return err
}

// Recovers and saves state summary for a given root if the root has a block in the DB.
func recoverStateSummary(ctx context.Context, tx *bolt.Tx, root []byte) error {
	blkBucket := tx.Bucket(blocksBucket)
	blkEnc := blkBucket.Get(root)
	if blkEnc == nil {
		return fmt.Errorf("nil block, root: %#x", bytesutil.Trunc(root))
	}
	blk, err := unmarshalBlock(ctx, blkEnc)
	if err != nil {
		return errors.Wrapf(err, "Could not unmarshal block: %#x", bytesutil.Trunc(root))
	}
	summaryEnc, err := encode(ctx, &ethpb.StateSummary{
		Slot: blk.Block().Slot(),
		Root: root,
	})
	if err != nil {
		return err
	}
	summaryBucket := tx.Bucket(stateBucket)
	return summaryBucket.Put(root, summaryEnc)
}
