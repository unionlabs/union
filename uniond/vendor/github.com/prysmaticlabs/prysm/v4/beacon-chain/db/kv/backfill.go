package kv

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/proto/dbval"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
	"google.golang.org/protobuf/proto"
)

// SaveBackfillStatus encodes the given BackfillStatus protobuf struct and writes it to a single key in the db.
// This value is used by the backfill service to keep track of the range of blocks that need to be synced. It is also used by the
// code that serves blocks or regenerates states to keep track of what range of blocks are available.
func (s *Store) SaveBackfillStatus(ctx context.Context, bf *dbval.BackfillStatus) error {
	_, span := trace.StartSpan(ctx, "BeaconDB.SaveBackfillStatus")
	defer span.End()
	bfb, err := proto.Marshal(bf)
	if err != nil {
		return err
	}
	return s.db.Update(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(blocksBucket)
		return bucket.Put(backfillStatusKey, bfb)
	})
}

// BackfillStatus retrieves the most recently saved version of the BackfillStatus protobuf struct.
// This is used to persist information about backfill status across restarts.
func (s *Store) BackfillStatus(ctx context.Context) (*dbval.BackfillStatus, error) {
	_, span := trace.StartSpan(ctx, "BeaconDB.BackfillStatus")
	defer span.End()
	bf := &dbval.BackfillStatus{}
	err := s.db.View(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(blocksBucket)
		bs := bucket.Get(backfillStatusKey)
		if len(bs) == 0 {
			return errors.Wrap(ErrNotFound, "BackfillStatus not found")
		}
		return proto.Unmarshal(bs, bf)
	})
	return bf, err
}
