package bbolt_test

import (
	crand "crypto/rand"
	"math/rand"
	"os"
	"path/filepath"
	"testing"

	"go.etcd.io/bbolt"
	"go.etcd.io/bbolt/errors"
	"go.etcd.io/bbolt/internal/btesting"

	"github.com/stretchr/testify/require"
)

func TestTx_MoveBucket(t *testing.T) {
	testCases := []struct {
		name                    string
		srcBucketPath           []string
		dstBucketPath           []string
		bucketToMove            string
		bucketExistInSrc        bool
		bucketExistInDst        bool
		hasIncompatibleKeyInSrc bool
		hasIncompatibleKeyInDst bool
		expectedErr             error
	}{
		// normal cases
		{
			name:                    "normal case",
			srcBucketPath:           []string{"sb1", "sb2"},
			dstBucketPath:           []string{"db1", "db2"},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        true,
			bucketExistInDst:        false,
			hasIncompatibleKeyInSrc: false,
			hasIncompatibleKeyInDst: false,
			expectedErr:             nil,
		},
		{
			name:                    "the source and target bucket share the same grandparent",
			srcBucketPath:           []string{"grandparent", "sb2"},
			dstBucketPath:           []string{"grandparent", "db2"},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        true,
			bucketExistInDst:        false,
			hasIncompatibleKeyInSrc: false,
			hasIncompatibleKeyInDst: false,
			expectedErr:             nil,
		},
		{
			name:                    "bucketToMove is a top level bucket",
			srcBucketPath:           []string{},
			dstBucketPath:           []string{"db1", "db2"},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        true,
			bucketExistInDst:        false,
			hasIncompatibleKeyInSrc: false,
			hasIncompatibleKeyInDst: false,
			expectedErr:             nil,
		},
		{
			name:                    "convert bucketToMove to a top level bucket",
			srcBucketPath:           []string{"sb1", "sb2"},
			dstBucketPath:           []string{},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        true,
			bucketExistInDst:        false,
			hasIncompatibleKeyInSrc: false,
			hasIncompatibleKeyInDst: false,
			expectedErr:             nil,
		},
		// negative cases
		{
			name:                    "bucketToMove not exist in source bucket",
			srcBucketPath:           []string{"sb1", "sb2"},
			dstBucketPath:           []string{"db1", "db2"},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        false,
			bucketExistInDst:        false,
			hasIncompatibleKeyInSrc: false,
			hasIncompatibleKeyInDst: false,
			expectedErr:             errors.ErrBucketNotFound,
		},
		{
			name:                    "bucketToMove exist in target bucket",
			srcBucketPath:           []string{"sb1", "sb2"},
			dstBucketPath:           []string{"db1", "db2"},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        true,
			bucketExistInDst:        true,
			hasIncompatibleKeyInSrc: false,
			hasIncompatibleKeyInDst: false,
			expectedErr:             errors.ErrBucketExists,
		},
		{
			name:                    "incompatible key exist in source bucket",
			srcBucketPath:           []string{"sb1", "sb2"},
			dstBucketPath:           []string{"db1", "db2"},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        false,
			bucketExistInDst:        false,
			hasIncompatibleKeyInSrc: true,
			hasIncompatibleKeyInDst: false,
			expectedErr:             errors.ErrIncompatibleValue,
		},
		{
			name:                    "incompatible key exist in target bucket",
			srcBucketPath:           []string{"sb1", "sb2"},
			dstBucketPath:           []string{"db1", "db2"},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        true,
			bucketExistInDst:        false,
			hasIncompatibleKeyInSrc: false,
			hasIncompatibleKeyInDst: true,
			expectedErr:             errors.ErrIncompatibleValue,
		},
		{
			name:                    "the source and target are the same bucket",
			srcBucketPath:           []string{"sb1", "sb2"},
			dstBucketPath:           []string{"sb1", "sb2"},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        true,
			bucketExistInDst:        false,
			hasIncompatibleKeyInSrc: false,
			hasIncompatibleKeyInDst: false,
			expectedErr:             errors.ErrSameBuckets,
		},
		{
			name:                    "both the source and target are the root bucket",
			srcBucketPath:           []string{},
			dstBucketPath:           []string{},
			bucketToMove:            "bucketToMove",
			bucketExistInSrc:        true,
			bucketExistInDst:        false,
			hasIncompatibleKeyInSrc: false,
			hasIncompatibleKeyInDst: false,
			expectedErr:             errors.ErrSameBuckets,
		},
	}

	for _, tc := range testCases {

		t.Run(tc.name, func(*testing.T) {
			db := btesting.MustCreateDBWithOption(t, &bbolt.Options{PageSize: 4096})

			dumpBucketBeforeMoving := filepath.Join(t.TempDir(), "dbBeforeMove")
			dumpBucketAfterMoving := filepath.Join(t.TempDir(), "dbAfterMove")

			t.Log("Creating sample db and populate some data")
			err := db.Update(func(tx *bbolt.Tx) error {
				srcBucket := prepareBuckets(t, tx, tc.srcBucketPath...)
				dstBucket := prepareBuckets(t, tx, tc.dstBucketPath...)

				if tc.bucketExistInSrc {
					_ = createBucketAndPopulateData(t, tx, srcBucket, tc.bucketToMove)
				}

				if tc.bucketExistInDst {
					_ = createBucketAndPopulateData(t, tx, dstBucket, tc.bucketToMove)
				}

				if tc.hasIncompatibleKeyInSrc {
					putErr := srcBucket.Put([]byte(tc.bucketToMove), []byte("bar"))
					require.NoError(t, putErr)
				}

				if tc.hasIncompatibleKeyInDst {
					putErr := dstBucket.Put([]byte(tc.bucketToMove), []byte("bar"))
					require.NoError(t, putErr)
				}

				return nil
			})
			require.NoError(t, err)

			t.Log("Moving bucket")
			err = db.Update(func(tx *bbolt.Tx) error {
				srcBucket := prepareBuckets(t, tx, tc.srcBucketPath...)
				dstBucket := prepareBuckets(t, tx, tc.dstBucketPath...)

				if tc.expectedErr == nil {
					t.Logf("Dump the bucket to %s before moving it", dumpBucketBeforeMoving)
					bk := openBucket(tx, srcBucket, tc.bucketToMove)
					dumpErr := dumpBucket([]byte(tc.bucketToMove), bk, dumpBucketBeforeMoving)
					require.NoError(t, dumpErr)
				}

				mErr := tx.MoveBucket([]byte(tc.bucketToMove), srcBucket, dstBucket)
				require.Equal(t, tc.expectedErr, mErr)

				if tc.expectedErr == nil {
					t.Logf("Dump the bucket to %s after moving it", dumpBucketAfterMoving)
					bk := openBucket(tx, dstBucket, tc.bucketToMove)
					dumpErr := dumpBucket([]byte(tc.bucketToMove), bk, dumpBucketAfterMoving)
					require.NoError(t, dumpErr)
				}

				return nil
			})
			require.NoError(t, err)

			// skip assertion if failure expected
			if tc.expectedErr != nil {
				return
			}

			t.Log("Verifying the bucket should be identical before and after being moved")
			dataBeforeMove, err := os.ReadFile(dumpBucketBeforeMoving)
			require.NoError(t, err)
			dataAfterMove, err := os.ReadFile(dumpBucketAfterMoving)
			require.NoError(t, err)
			require.Equal(t, dataBeforeMove, dataAfterMove)
		})
	}
}

func TestBucket_MoveBucket_DiffDB(t *testing.T) {
	srcBucketPath := []string{"sb1", "sb2"}
	dstBucketPath := []string{"db1", "db2"}
	bucketToMove := "bucketToMove"

	var srcBucket *bbolt.Bucket

	t.Log("Creating source bucket and populate some data")
	srcDB := btesting.MustCreateDBWithOption(t, &bbolt.Options{PageSize: 4096})
	err := srcDB.Update(func(tx *bbolt.Tx) error {
		srcBucket = prepareBuckets(t, tx, srcBucketPath...)
		return nil
	})
	require.NoError(t, err)
	defer func() {
		require.NoError(t, srcDB.Close())
	}()

	t.Log("Creating target bucket and populate some data")
	dstDB := btesting.MustCreateDBWithOption(t, &bbolt.Options{PageSize: 4096})
	err = dstDB.Update(func(tx *bbolt.Tx) error {
		prepareBuckets(t, tx, dstBucketPath...)
		return nil
	})
	require.NoError(t, err)
	defer func() {
		require.NoError(t, dstDB.Close())
	}()

	t.Log("Reading source bucket in a separate RWTx")
	sTx, sErr := srcDB.Begin(true)
	require.NoError(t, sErr)
	defer func() {
		require.NoError(t, sTx.Rollback())
	}()
	srcBucket = prepareBuckets(t, sTx, srcBucketPath...)

	t.Log("Moving the sub-bucket in a separate RWTx")
	err = dstDB.Update(func(tx *bbolt.Tx) error {
		dstBucket := prepareBuckets(t, tx, dstBucketPath...)
		mErr := srcBucket.MoveBucket([]byte(bucketToMove), dstBucket)
		require.Equal(t, errors.ErrDifferentDB, mErr)

		return nil
	})
	require.NoError(t, err)
}

func TestBucket_MoveBucket_DiffTx(t *testing.T) {
	testCases := []struct {
		name            string
		srcBucketPath   []string
		dstBucketPath   []string
		isSrcReadonlyTx bool
		isDstReadonlyTx bool
		bucketToMove    string
		expectedErr     error
	}{
		{
			name:            "src is RWTx and target is RTx",
			srcBucketPath:   []string{"sb1", "sb2"},
			dstBucketPath:   []string{"db1", "db2"},
			isSrcReadonlyTx: true,
			isDstReadonlyTx: false,
			bucketToMove:    "bucketToMove",
			expectedErr:     errors.ErrTxNotWritable,
		},
		{
			name:            "src is RTx and target is RWTx",
			srcBucketPath:   []string{"sb1", "sb2"},
			dstBucketPath:   []string{"db1", "db2"},
			isSrcReadonlyTx: false,
			isDstReadonlyTx: true,
			bucketToMove:    "bucketToMove",
			expectedErr:     errors.ErrTxNotWritable,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			var srcBucket *bbolt.Bucket
			var dstBucket *bbolt.Bucket

			t.Log("Creating source and target buckets and populate some data")
			db := btesting.MustCreateDBWithOption(t, &bbolt.Options{PageSize: 4096})
			err := db.Update(func(tx *bbolt.Tx) error {
				srcBucket = prepareBuckets(t, tx, tc.srcBucketPath...)
				dstBucket = prepareBuckets(t, tx, tc.dstBucketPath...)
				return nil
			})
			require.NoError(t, err)
			defer func() {
				require.NoError(t, db.Close())
			}()

			t.Log("Opening source bucket in a separate Tx")
			sTx, sErr := db.Begin(tc.isSrcReadonlyTx)
			require.NoError(t, sErr)
			defer func() {
				require.NoError(t, sTx.Rollback())
			}()
			srcBucket = prepareBuckets(t, sTx, tc.srcBucketPath...)

			t.Log("Opening target bucket in a separate Tx")
			dTx, dErr := db.Begin(tc.isDstReadonlyTx)
			require.NoError(t, dErr)
			defer func() {
				require.NoError(t, dTx.Rollback())
			}()
			dstBucket = prepareBuckets(t, dTx, tc.dstBucketPath...)

			t.Log("Moving the sub-bucket")
			err = db.View(func(tx *bbolt.Tx) error {
				mErr := srcBucket.MoveBucket([]byte(tc.bucketToMove), dstBucket)
				require.Equal(t, tc.expectedErr, mErr)

				return nil
			})
			require.NoError(t, err)
		})
	}
}

// prepareBuckets opens the bucket chain. For each bucket in the chain,
// open it if existed, otherwise create it and populate sample data.
func prepareBuckets(t testing.TB, tx *bbolt.Tx, buckets ...string) *bbolt.Bucket {
	var bk *bbolt.Bucket

	for _, key := range buckets {
		if childBucket := openBucket(tx, bk, key); childBucket == nil {
			bk = createBucketAndPopulateData(t, tx, bk, key)
		} else {
			bk = childBucket
		}
	}
	return bk
}

func openBucket(tx *bbolt.Tx, bk *bbolt.Bucket, bucketToOpen string) *bbolt.Bucket {
	if bk == nil {
		return tx.Bucket([]byte(bucketToOpen))
	}
	return bk.Bucket([]byte(bucketToOpen))
}

func createBucketAndPopulateData(t testing.TB, tx *bbolt.Tx, bk *bbolt.Bucket, bucketName string) *bbolt.Bucket {
	if bk == nil {
		newBucket, err := tx.CreateBucket([]byte(bucketName))
		require.NoError(t, err, "failed to create bucket %s", bucketName)
		populateSampleDataInBucket(t, newBucket, rand.Intn(4096))
		return newBucket
	}

	newBucket, err := bk.CreateBucket([]byte(bucketName))
	require.NoError(t, err, "failed to create bucket %s", bucketName)
	populateSampleDataInBucket(t, newBucket, rand.Intn(4096))
	return newBucket
}

func populateSampleDataInBucket(t testing.TB, bk *bbolt.Bucket, n int) {
	var min, max = 1, 1024

	for i := 0; i < n; i++ {
		// generate rand key/value length
		keyLength := rand.Intn(max-min) + min
		valLength := rand.Intn(max-min) + min

		keyData := make([]byte, keyLength)
		valData := make([]byte, valLength)

		_, err := crand.Read(keyData)
		require.NoError(t, err)

		_, err = crand.Read(valData)
		require.NoError(t, err)

		err = bk.Put(keyData, valData)
		require.NoError(t, err)
	}
}
