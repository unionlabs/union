package kv

import (
	"bytes"
	"context"
	"fmt"

	bolt "go.etcd.io/bbolt"
)

// SaveGenesisValidatorsRoot saves the genesis validators root to db.
func (s *Store) SaveGenesisValidatorsRoot(_ context.Context, genValRoot []byte) error {
	err := s.db.Update(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(genesisInfoBucket)
		enc := bkt.Get(genesisValidatorsRootKey)
		if len(enc) != 0 && !bytes.Equal(enc, genValRoot) {
			return fmt.Errorf("cannot overwrite existing genesis validators root: %#x", enc)
		}
		return bkt.Put(genesisValidatorsRootKey, genValRoot)
	})
	return err
}

// GenesisValidatorsRoot retrieves the genesis validators root from db.
func (s *Store) GenesisValidatorsRoot(_ context.Context) ([]byte, error) {
	var genValRoot []byte
	err := s.db.View(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(genesisInfoBucket)
		enc := bkt.Get(genesisValidatorsRootKey)
		if len(enc) == 0 {
			return nil
		}
		genValRoot = enc
		return nil
	})
	return genValRoot, err
}
