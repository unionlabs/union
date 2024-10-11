package kv

import (
	"context"
	"fmt"
	"path"
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
)

const backupsDirectoryName = "backups"

// Backup the database to the datadir backup directory.
// Example for backup: $DATADIR/backups/prysm_validatordb_1029019.backup
func (s *Store) Backup(ctx context.Context, outputDir string, permissionOverride bool) error {
	_, span := trace.StartSpan(ctx, "ValidatorDB.Backup")
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
	// Ensure the backups directory exists.
	if err := file.HandleBackupDir(backupsDir, permissionOverride); err != nil {
		return err
	}
	backupPath := path.Join(backupsDir, fmt.Sprintf("prysm_validatordb_%d.backup", time.Now().Unix()))
	log.WithField("backup", backupPath).Info("Writing backup database")

	copyDB, err := bolt.Open(
		backupPath,
		params.BeaconIoConfig().ReadWritePermissions,
		&bolt.Options{Timeout: params.BeaconIoConfig().BoltTimeout},
	)
	if err != nil {
		return err
	}
	defer func() {
		if err := copyDB.Close(); err != nil {
			log.WithError(err).Error("Failed to close backup database")
		}
	}()

	return s.db.View(func(tx *bolt.Tx) error {
		return tx.ForEach(func(name []byte, b *bolt.Bucket) error {
			log.Debugf("Copying bucket %s\n with %d keys", name, b.Stats().KeyN)
			return copyDB.Update(func(tx2 *bolt.Tx) error {
				b2, err := tx2.CreateBucketIfNotExists(name)
				if err != nil {
					return err
				}
				return b.ForEach(createNestedBuckets(b, b2, b2.Put))
			})
		})
	})
}

// Walks through each buckets and looks out for nested buckets so that
// the backup db also includes them.
func createNestedBuckets(srcBucket, dstBucket *bolt.Bucket, fn func(k, v []byte) error) func(k, v []byte) error {
	return func(k, v []byte) error {
		bkt := srcBucket.Bucket(k)
		if bkt != nil {
			b2, err := dstBucket.CreateBucketIfNotExists(k)
			if err != nil {
				return err
			}
			return bkt.ForEach(createNestedBuckets(bkt, b2, b2.Put))
		}
		return fn(k, v)
	}
}
