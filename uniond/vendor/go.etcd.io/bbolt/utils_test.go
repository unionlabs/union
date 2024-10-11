package bbolt_test

import (
	bolt "go.etcd.io/bbolt"
	"go.etcd.io/bbolt/internal/common"
)

// `dumpBucket` dumps all the data, including both key/value data
// and child buckets, from the source bucket into the target db file.
func dumpBucket(srcBucketName []byte, srcBucket *bolt.Bucket, dstFilename string) error {
	common.Assert(len(srcBucketName) != 0, "source bucket name can't be empty")
	common.Assert(srcBucket != nil, "the source bucket can't be nil")
	common.Assert(len(dstFilename) != 0, "the target file path can't be empty")

	dstDB, err := bolt.Open(dstFilename, 0600, nil)
	if err != nil {
		return err
	}
	defer dstDB.Close()

	return dstDB.Update(func(tx *bolt.Tx) error {
		dstBucket, err := tx.CreateBucket(srcBucketName)
		if err != nil {
			return err
		}
		return cloneBucket(srcBucket, dstBucket)
	})
}

func cloneBucket(src *bolt.Bucket, dst *bolt.Bucket) error {
	return src.ForEach(func(k, v []byte) error {
		if v == nil {
			srcChild := src.Bucket(k)
			dstChild, err := dst.CreateBucket(k)
			if err != nil {
				return err
			}
			if err = dstChild.SetSequence(srcChild.Sequence()); err != nil {
				return err
			}

			return cloneBucket(srcChild, dstChild)
		}

		return dst.Put(k, v)
	})
}
