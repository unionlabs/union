//go:build !windows
// +build !windows

package bbolt_test

import (
	"fmt"
	"testing"

	"golang.org/x/sys/unix"

	bolt "go.etcd.io/bbolt"
	"go.etcd.io/bbolt/internal/btesting"
)

func TestMlock_DbOpen(t *testing.T) {
	// 32KB
	skipOnMemlockLimitBelow(t, 32*1024)

	btesting.MustCreateDBWithOption(t, &bolt.Options{Mlock: true})
}

// Test change between "empty" (16KB) and "non-empty" db
func TestMlock_DbCanGrow_Small(t *testing.T) {
	// 32KB
	skipOnMemlockLimitBelow(t, 32*1024)

	db := btesting.MustCreateDBWithOption(t, &bolt.Options{Mlock: true})

	if err := db.Update(func(tx *bolt.Tx) error {
		b, err := tx.CreateBucketIfNotExists([]byte("bucket"))
		if err != nil {
			t.Fatal(err)
		}

		key := []byte("key")
		value := []byte("value")
		if err := b.Put(key, value); err != nil {
			t.Fatal(err)
		}

		return nil
	}); err != nil {
		t.Fatal(err)
	}

}

// Test crossing of 16MB (AllocSize) of db size
func TestMlock_DbCanGrow_Big(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test in short mode")
	}

	// 32MB
	skipOnMemlockLimitBelow(t, 32*1024*1024)

	chunksBefore := 64
	chunksAfter := 64

	db := btesting.MustCreateDBWithOption(t, &bolt.Options{Mlock: true})

	for chunk := 0; chunk < chunksBefore; chunk++ {
		insertChunk(t, db, chunk)
	}
	dbSize := fileSize(db.Path())

	for chunk := 0; chunk < chunksAfter; chunk++ {
		insertChunk(t, db, chunksBefore+chunk)
	}
	newDbSize := fileSize(db.Path())

	if newDbSize <= dbSize {
		t.Errorf("db didn't grow: %v <= %v", newDbSize, dbSize)
	}
}

func insertChunk(t *testing.T, db *btesting.DB, chunkId int) {
	chunkSize := 1024

	if err := db.Update(func(tx *bolt.Tx) error {
		b, err := tx.CreateBucketIfNotExists([]byte("bucket"))
		if err != nil {
			t.Fatal(err)
		}

		for i := 0; i < chunkSize; i++ {
			key := []byte(fmt.Sprintf("key-%d-%d", chunkId, i))
			value := []byte("value")
			if err := b.Put(key, value); err != nil {
				t.Fatal(err)
			}
		}

		return nil
	}); err != nil {
		t.Fatal(err)
	}
}

// Main reason for this check is travis limiting mlockable memory to 64KB
// https://github.com/travis-ci/travis-ci/issues/2462
func skipOnMemlockLimitBelow(t *testing.T, memlockLimitRequest uint64) {
	var info unix.Rlimit
	if err := unix.Getrlimit(unix.RLIMIT_MEMLOCK, &info); err != nil {
		t.Fatal(err)
	}

	if info.Cur < memlockLimitRequest {
		t.Skipf(
			"skipping as RLIMIT_MEMLOCK is insufficient: %v < %v",
			info.Cur,
			memlockLimitRequest,
		)
	}
}
