// Package slasherkv defines a bolt-db, key-value store implementation
// of the slasher database interface for Prysm.
package slasherkv

import (
	"context"
	"os"
	"path"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/iface"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	bolt "go.etcd.io/bbolt"
)

var _ iface.SlasherDatabase = (*Store)(nil)

const (
	// DatabaseFileName is the name of the beacon node database.
	DatabaseFileName = "slasher.db"
	boltAllocSize    = 8 * 1024 * 1024
	// Specifies the initial mmap size of bolt.
	mmapSize = 536870912
)

// Store defines an implementation of the Prysm Database interface
// using BoltDB as the underlying persistent kv-store for Ethereum consensus.
type Store struct {
	db           *bolt.DB
	databasePath string
	ctx          context.Context
}

// NewKVStore initializes a new boltDB key-value store at the directory
// path specified, creates the kv-buckets based on the schema, and stores
// an open connection db object as a property of the Store struct.
func NewKVStore(ctx context.Context, dirPath string) (*Store, error) {
	hasDir, err := file.HasDir(dirPath)
	if err != nil {
		return nil, err
	}
	if !hasDir {
		if err := file.MkdirAll(dirPath); err != nil {
			return nil, err
		}
	}
	datafile := path.Join(dirPath, DatabaseFileName)
	boltDB, err := bolt.Open(
		datafile,
		params.BeaconIoConfig().ReadWritePermissions,
		&bolt.Options{
			Timeout:         1 * time.Second,
			InitialMmapSize: mmapSize,
		},
	)
	if err != nil {
		if errors.Is(err, bolt.ErrTimeout) {
			return nil, errors.New("cannot obtain database lock, database may be in use by another process")
		}
		return nil, err
	}
	boltDB.AllocSize = boltAllocSize
	kv := &Store{
		db:           boltDB,
		databasePath: dirPath,
		ctx:          ctx,
	}

	if err := kv.db.Update(func(tx *bolt.Tx) error {
		return createBuckets(
			tx,
			// Slasher buckets.
			attestedEpochsByValidator,
			attestationRecordsBucket,
			attestationDataRootsBucket,
			proposalRecordsBucket,
			slasherChunksBucket,
		)
	}); err != nil {
		return nil, err
	}

	return kv, err
}

// ClearDB removes the previously stored database in the data directory.
func (s *Store) ClearDB() error {
	if _, err := os.Stat(s.databasePath); os.IsNotExist(err) {
		return nil
	}
	if err := os.Remove(path.Join(s.databasePath, DatabaseFileName)); err != nil {
		return errors.Wrap(err, "could not remove database file")
	}
	return nil
}

// Close closes the underlying BoltDB database.
func (s *Store) Close() error {
	return s.db.Close()
}

// DatabasePath at which this database writes files.
func (s *Store) DatabasePath() string {
	return s.databasePath
}

func createBuckets(tx *bolt.Tx, buckets ...[]byte) error {
	for _, bucket := range buckets {
		if _, err := tx.CreateBucketIfNotExists(bucket); err != nil {
			return err
		}
	}
	return nil
}
