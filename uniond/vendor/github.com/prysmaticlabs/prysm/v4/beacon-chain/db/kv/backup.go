package kv

import (
	"context"
	"fmt"
	"path"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
)

const backupsDirectoryName = "backups"

// Backup the database to the datadir backup directory.
// Example for backup at slot 345: $DATADIR/backups/prysm_beacondb_at_slot_0000345.backup
func (s *Store) Backup(ctx context.Context, outputDir string, permissionOverride bool) error {
	ctx, span := trace.StartSpan(ctx, "BeaconDB.Backup")
	defer span.End()

	var backupsDir string
	var err error
	if outputDir != "" {
		backupsDir, err = file.ExpandPath(outputDir)
		if err != nil {
			return err
		}
	} else {
		backupsDir = path.Join(s.databasePath, backupsDirectoryName)
	}
	head, err := s.HeadBlock(ctx)
	if err != nil {
		return err
	}
	if err := blocks.BeaconBlockIsNil(head); err != nil {
		return err
	}
	// Ensure the backups directory exists.
	if err := file.HandleBackupDir(backupsDir, permissionOverride); err != nil {
		return err
	}
	backupPath := path.Join(backupsDir, fmt.Sprintf("prysm_beacondb_at_slot_%07d.backup", head.Block().Slot()))
	log.WithField("backup", backupPath).Info("Writing backup database.")

	copyDB, err := bolt.Open(
		backupPath,
		params.BeaconIoConfig().ReadWritePermissions,
		&bolt.Options{NoSync: true, Timeout: params.BeaconIoConfig().BoltTimeout, FreelistType: bolt.FreelistMapType},
	)
	if err != nil {
		return err
	}
	copyDB.AllocSize = boltAllocSize

	defer func() {
		if err := copyDB.Close(); err != nil {
			log.WithError(err).Error("Failed to close backup database")
		}
	}()
	// Prefetch all keys of buckets, and inner keys in a
	// bucket to use less memory usage when backing up.
	var bucketKeys [][]byte
	bucketMap := make(map[string][][]byte)
	err = s.db.View(func(tx *bolt.Tx) error {
		return tx.ForEach(func(name []byte, b *bolt.Bucket) error {
			newName := make([]byte, len(name))
			copy(newName, name)
			bucketKeys = append(bucketKeys, newName)
			var innerKeys [][]byte
			err := b.ForEach(func(k, v []byte) error {
				if k == nil {
					return nil
				}
				nKey := make([]byte, len(k))
				copy(nKey, k)
				innerKeys = append(innerKeys, nKey)
				return nil
			})
			if err != nil {
				return err
			}
			bucketMap[string(newName)] = innerKeys
			return nil
		})
	})
	if err != nil {
		return err
	}
	// Utilize much smaller writes, compared to
	// writing for a whole bucket in a single transaction. Also
	// prevent long-running read transactions, as Bolt doesn't
	// handle those well.
	for _, k := range bucketKeys {
		log.Debugf("Copying bucket %s\n", k)
		innerKeys := bucketMap[string(k)]
		for _, ik := range innerKeys {
			err = s.db.View(func(tx *bolt.Tx) error {
				bkt := tx.Bucket(k)
				return copyDB.Update(func(tx2 *bolt.Tx) error {
					b2, err := tx2.CreateBucketIfNotExists(k)
					if err != nil {
						return err
					}
					return b2.Put(ik, bkt.Get(ik))
				})
			})
			if err != nil {
				return err
			}
		}
	}
	// Re-enable sync to allow bolt to fsync
	// again.
	copyDB.NoSync = false
	return nil
}
