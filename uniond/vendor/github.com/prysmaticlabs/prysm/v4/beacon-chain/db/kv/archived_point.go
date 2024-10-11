package kv

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
)

// LastArchivedSlot from the db.
func (s *Store) LastArchivedSlot(ctx context.Context) (primitives.Slot, error) {
	_, span := trace.StartSpan(ctx, "BeaconDB.LastArchivedSlot")
	defer span.End()
	var index primitives.Slot
	err := s.db.View(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(stateSlotIndicesBucket)
		b, _ := bkt.Cursor().Last()
		index = bytesutil.BytesToSlotBigEndian(b)
		return nil
	})

	return index, err
}

// LastArchivedRoot from the db.
func (s *Store) LastArchivedRoot(ctx context.Context) [32]byte {
	_, span := trace.StartSpan(ctx, "BeaconDB.LastArchivedRoot")
	defer span.End()

	var blockRoot []byte
	if err := s.db.View(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(stateSlotIndicesBucket)
		_, blockRoot = bkt.Cursor().Last()
		return nil
	}); err != nil { // This view never returns an error, but we'll handle anyway for sanity.
		panic(err)
	}

	return bytesutil.ToBytes32(blockRoot)
}

// ArchivedPointRoot returns the block root of an archived point from the DB.
// This is essential for cold state management and to restore a cold state.
func (s *Store) ArchivedPointRoot(ctx context.Context, slot primitives.Slot) [32]byte {
	_, span := trace.StartSpan(ctx, "BeaconDB.ArchivedPointRoot")
	defer span.End()

	var blockRoot []byte
	if err := s.db.View(func(tx *bolt.Tx) error {
		bucket := tx.Bucket(stateSlotIndicesBucket)
		blockRoot = bucket.Get(bytesutil.SlotToBytesBigEndian(slot))
		return nil
	}); err != nil { // This view never returns an error, but we'll handle anyway for sanity.
		panic(err)
	}

	return bytesutil.ToBytes32(blockRoot)
}

// HasArchivedPoint returns true if an archived point exists in DB.
func (s *Store) HasArchivedPoint(ctx context.Context, slot primitives.Slot) bool {
	_, span := trace.StartSpan(ctx, "BeaconDB.HasArchivedPoint")
	defer span.End()
	var exists bool
	if err := s.db.View(func(tx *bolt.Tx) error {
		iBucket := tx.Bucket(stateSlotIndicesBucket)
		exists = iBucket.Get(bytesutil.SlotToBytesBigEndian(slot)) != nil
		return nil
	}); err != nil { // This view never returns an error, but we'll handle anyway for sanity.
		panic(err)
	}
	return exists
}
