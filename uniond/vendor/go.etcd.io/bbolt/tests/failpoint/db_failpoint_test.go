package failpoint

import (
	"fmt"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	bolt "go.etcd.io/bbolt"
	"go.etcd.io/bbolt/errors"
	"go.etcd.io/bbolt/internal/btesting"
	gofail "go.etcd.io/gofail/runtime"
)

func TestFailpoint_MapFail(t *testing.T) {
	err := gofail.Enable("mapError", `return("map somehow failed")`)
	require.NoError(t, err)
	defer func() {
		err = gofail.Disable("mapError")
		require.NoError(t, err)
	}()

	f := filepath.Join(t.TempDir(), "db")
	_, err = bolt.Open(f, 0600, nil)
	require.Error(t, err)
	require.ErrorContains(t, err, "map somehow failed")
}

// ensures when munmap fails, the flock is unlocked
func TestFailpoint_UnmapFail_DbClose(t *testing.T) {
	//unmap error on db close
	//we need to open the db first, and then enable the error.
	//otherwise the db cannot be opened.
	f := filepath.Join(t.TempDir(), "db")

	err := gofail.Enable("unmapError", `return("unmap somehow failed")`)
	require.NoError(t, err)
	_, err = bolt.Open(f, 0600, nil)
	require.Error(t, err)
	require.ErrorContains(t, err, "unmap somehow failed")
	//disable the error, and try to reopen the db
	err = gofail.Disable("unmapError")
	require.NoError(t, err)

	db, err := bolt.Open(f, 0600, &bolt.Options{Timeout: 30 * time.Second})
	require.NoError(t, err)
	err = db.Close()
	require.NoError(t, err)
}

func TestFailpoint_mLockFail(t *testing.T) {
	err := gofail.Enable("mlockError", `return("mlock somehow failed")`)
	require.NoError(t, err)

	f := filepath.Join(t.TempDir(), "db")
	_, err = bolt.Open(f, 0600, &bolt.Options{Mlock: true})
	require.Error(t, err)
	require.ErrorContains(t, err, "mlock somehow failed")

	// It should work after disabling the failpoint.
	err = gofail.Disable("mlockError")
	require.NoError(t, err)

	_, err = bolt.Open(f, 0600, &bolt.Options{Mlock: true})
	require.NoError(t, err)
}

func TestFailpoint_mLockFail_When_remap(t *testing.T) {
	db := btesting.MustCreateDB(t)
	db.Mlock = true

	err := gofail.Enable("mlockError", `return("mlock somehow failed in allocate")`)
	require.NoError(t, err)

	err = db.Fill([]byte("data"), 1, 10000,
		func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
		func(tx int, k int) []byte { return make([]byte, 100) },
	)

	require.Error(t, err)
	require.ErrorContains(t, err, "mlock somehow failed in allocate")

	// It should work after disabling the failpoint.
	err = gofail.Disable("mlockError")
	require.NoError(t, err)
	db.MustClose()
	db.MustReopen()

	err = db.Fill([]byte("data"), 1, 10000,
		func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
		func(tx int, k int) []byte { return make([]byte, 100) },
	)

	require.NoError(t, err)
}

func TestFailpoint_ResizeFileFail(t *testing.T) {
	db := btesting.MustCreateDB(t)

	err := gofail.Enable("resizeFileError", `return("resizeFile somehow failed")`)
	require.NoError(t, err)

	err = db.Fill([]byte("data"), 1, 10000,
		func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
		func(tx int, k int) []byte { return make([]byte, 100) },
	)

	require.Error(t, err)
	require.ErrorContains(t, err, "resizeFile somehow failed")

	// It should work after disabling the failpoint.
	err = gofail.Disable("resizeFileError")
	require.NoError(t, err)
	db.MustClose()
	db.MustReopen()

	err = db.Fill([]byte("data"), 1, 10000,
		func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
		func(tx int, k int) []byte { return make([]byte, 100) },
	)

	require.NoError(t, err)
}

func TestFailpoint_LackOfDiskSpace(t *testing.T) {
	db := btesting.MustCreateDB(t)

	err := gofail.Enable("lackOfDiskSpace", `return("grow somehow failed")`)
	require.NoError(t, err)

	tx, err := db.Begin(true)
	require.NoError(t, err)

	err = tx.Commit()
	require.Error(t, err)
	require.ErrorContains(t, err, "grow somehow failed")

	err = tx.Rollback()
	require.Error(t, err)
	require.ErrorIs(t, err, errors.ErrTxClosed)

	// It should work after disabling the failpoint.
	err = gofail.Disable("lackOfDiskSpace")
	require.NoError(t, err)

	tx, err = db.Begin(true)
	require.NoError(t, err)

	err = tx.Commit()
	require.NoError(t, err)

	err = tx.Rollback()
	require.Error(t, err)
	require.ErrorIs(t, err, errors.ErrTxClosed)
}

// TestIssue72 reproduces issue 72.
//
// When bbolt is processing a `Put` invocation, the key might be concurrently
// updated by the application which calls the `Put` API (although it shouldn't).
// It might lead to a situation that bbolt use an old key to find a proper
// position to insert the key/value pair, but actually inserts a new key.
// Eventually it might break the rule that all keys should be sorted. In a
// worse case, it might cause page elements to point to already freed pages.
//
// REF: https://github.com/etcd-io/bbolt/issues/72
func TestIssue72(t *testing.T) {
	db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: 4096})

	bucketName := []byte(t.Name())
	err := db.Update(func(tx *bolt.Tx) error {
		_, txerr := tx.CreateBucket(bucketName)
		return txerr
	})
	require.NoError(t, err)

	// The layout is like:
	//
	//         +--+--+--+
	//  +------+1 |3 |10+---+
	//  |      +-++--+--+   |
	//  |         |         |
	//  |         |         |
	// +v-+--+   +v-+--+  +-v+--+--+
	// |1 |2 |   |3 |4 |  |10|11|12|
	// +--+--+   +--+--+  +--+--+--+
	//
	err = db.Update(func(tx *bolt.Tx) error {
		bk := tx.Bucket(bucketName)

		for _, id := range []int{1, 2, 3, 4, 10, 11, 12} {
			if txerr := bk.Put(idToBytes(id), make([]byte, 1000)); txerr != nil {
				return txerr
			}
		}
		return nil
	})
	require.NoError(t, err)

	require.NoError(t, gofail.Enable("beforeBucketPut", `sleep(5000)`))

	//         +--+--+--+
	//  +------+1 |3 |1 +---+
	//  |      +-++--+--+   |
	//  |         |         |
	//  |         |         |
	// +v-+--+   +v-+--+  +-v+--+--+--+
	// |1 |2 |   |3 |4 |  |1 |10|11|12|
	// +--+--+   +--+--+  +--+--+--+--+
	//
	key := idToBytes(13)
	updatedKey := idToBytes(1)
	err = db.Update(func(tx *bolt.Tx) error {
		bk := tx.Bucket(bucketName)

		go func() {
			time.Sleep(3 * time.Second)
			copy(key, updatedKey)
		}()
		return bk.Put(key, make([]byte, 100))
	})
	require.NoError(t, err)

	require.NoError(t, gofail.Disable("beforeBucketPut"))

	// bbolt inserts 100 into last branch page. Since there are two `1`
	// keys in branch, spill operation will update first `1` pointer and
	// then last one won't be updated and continues to point to freed page.
	//
	//
	//                  +--+--+--+
	//  +---------------+1 |3 |1 +---------+
	//  |               +--++-+--+         |
	//  |                   |              |
	//  |                   |              |
	//  |        +--+--+   +v-+--+   +-----v-----+
	//  |        |1 |2 |   |3 |4 |   |freed page |
	//  |        +--+--+   +--+--+   +-----------+
	//  |
	// +v-+--+--+--+---+
	// |1 |10|11|12|100|
	// +--+--+--+--+---+
	err = db.Update(func(tx *bolt.Tx) error {
		return tx.Bucket(bucketName).Put(idToBytes(100), make([]byte, 100))
	})
	require.NoError(t, err)

	defer func() {
		if r := recover(); r != nil {
			t.Logf("panic info:\n %v", r)
		}
	}()

	// Add more keys to ensure branch node to spill.
	err = db.Update(func(tx *bolt.Tx) error {
		bk := tx.Bucket(bucketName)

		for _, id := range []int{101, 102, 103, 104, 105} {
			if txerr := bk.Put(idToBytes(id), make([]byte, 1000)); txerr != nil {
				return txerr
			}
		}
		return nil
	})
	require.NoError(t, err)
}

func idToBytes(id int) []byte {
	return []byte(fmt.Sprintf("%010d", id))
}
