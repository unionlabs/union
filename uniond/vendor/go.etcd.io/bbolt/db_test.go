package bbolt_test

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"hash/fnv"
	"log"
	"math/rand"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"sync"
	"testing"
	"time"
	"unsafe"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	bolt "go.etcd.io/bbolt"
	berrors "go.etcd.io/bbolt/errors"
	"go.etcd.io/bbolt/internal/btesting"
)

// pageSize is the size of one page in the data file.
const pageSize = 4096

// pageHeaderSize is the size of a page header.
const pageHeaderSize = 16

// meta represents a simplified version of a database meta page for testing.
type meta struct {
	_       uint32
	version uint32
	_       uint32
	_       uint32
	_       [16]byte
	_       uint64
	pgid    uint64
	_       uint64
	_       uint64
}

// Ensure that a database can be opened without error.
func TestOpen(t *testing.T) {
	path := tempfile()
	defer os.RemoveAll(path)

	db, err := bolt.Open(path, 0600, nil)
	if err != nil {
		t.Fatal(err)
	} else if db == nil {
		t.Fatal("expected db")
	}

	if s := db.Path(); s != path {
		t.Fatalf("unexpected path: %s", s)
	}

	if err := db.Close(); err != nil {
		t.Fatal(err)
	}
}

// Regression validation for https://github.com/etcd-io/bbolt/pull/122.
// Tests multiple goroutines simultaneously opening a database.
func TestOpen_MultipleGoroutines(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test in short mode")
	}

	const (
		instances  = 30
		iterations = 30
	)
	path := tempfile()
	defer os.RemoveAll(path)
	var wg sync.WaitGroup
	errCh := make(chan error, iterations*instances)
	for iteration := 0; iteration < iterations; iteration++ {
		for instance := 0; instance < instances; instance++ {
			wg.Add(1)
			go func() {
				defer wg.Done()
				db, err := bolt.Open(path, 0600, nil)
				if err != nil {
					errCh <- err
					return
				}
				if err := db.Close(); err != nil {
					errCh <- err
					return
				}
			}()
		}
		wg.Wait()
	}
	close(errCh)
	for err := range errCh {
		if err != nil {
			t.Fatalf("error from inside goroutine: %v", err)
		}
	}
}

// Ensure that opening a database with a blank path returns an error.
func TestOpen_ErrPathRequired(t *testing.T) {
	_, err := bolt.Open("", 0600, nil)
	if err == nil {
		t.Fatalf("expected error")
	}
}

// Ensure that opening a database with a bad path returns an error.
func TestOpen_ErrNotExists(t *testing.T) {
	_, err := bolt.Open(filepath.Join(tempfile(), "bad-path"), 0600, nil)
	if err == nil {
		t.Fatal("expected error")
	}
}

// Ensure that opening a file that is not a Bolt database returns ErrInvalid.
func TestOpen_ErrInvalid(t *testing.T) {
	path := tempfile()
	defer os.RemoveAll(path)

	f, err := os.Create(path)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := fmt.Fprintln(f, "this is not a bolt database"); err != nil {
		t.Fatal(err)
	}
	if err := f.Close(); err != nil {
		t.Fatal(err)
	}

	if _, err := bolt.Open(path, 0600, nil); err != berrors.ErrInvalid {
		t.Fatalf("unexpected error: %s", err)
	}
}

// Ensure that opening a file with two invalid versions returns ErrVersionMismatch.
func TestOpen_ErrVersionMismatch(t *testing.T) {
	if pageSize != os.Getpagesize() {
		t.Skip("page size mismatch")
	}

	// Create empty database.
	db := btesting.MustCreateDB(t)
	path := db.Path()

	// Close database.
	if err := db.Close(); err != nil {
		t.Fatal(err)
	}

	// Read data file.
	buf, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}

	// Rewrite meta pages.
	meta0 := (*meta)(unsafe.Pointer(&buf[pageHeaderSize]))
	meta0.version++
	meta1 := (*meta)(unsafe.Pointer(&buf[pageSize+pageHeaderSize]))
	meta1.version++
	if err := os.WriteFile(path, buf, 0666); err != nil {
		t.Fatal(err)
	}

	// Reopen data file.
	if _, err := bolt.Open(path, 0600, nil); err != berrors.ErrVersionMismatch {
		t.Fatalf("unexpected error: %s", err)
	}
}

// Ensure that opening a file with two invalid checksums returns ErrChecksum.
func TestOpen_ErrChecksum(t *testing.T) {
	if pageSize != os.Getpagesize() {
		t.Skip("page size mismatch")
	}

	// Create empty database.
	db := btesting.MustCreateDB(t)
	path := db.Path()

	// Close database.
	if err := db.Close(); err != nil {
		t.Fatal(err)
	}

	// Read data file.
	buf, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}

	// Rewrite meta pages.
	meta0 := (*meta)(unsafe.Pointer(&buf[pageHeaderSize]))
	meta0.pgid++
	meta1 := (*meta)(unsafe.Pointer(&buf[pageSize+pageHeaderSize]))
	meta1.pgid++
	if err := os.WriteFile(path, buf, 0666); err != nil {
		t.Fatal(err)
	}

	// Reopen data file.
	if _, err := bolt.Open(path, 0600, nil); err != berrors.ErrChecksum {
		t.Fatalf("unexpected error: %s", err)
	}
}

// Ensure that it can read the page size from the second meta page if the first one is invalid.
// The page size is expected to be the OS's page size in this case.
func TestOpen_ReadPageSize_FromMeta1_OS(t *testing.T) {
	// Create empty database.
	db := btesting.MustCreateDB(t)
	path := db.Path()
	// Close the database
	db.MustClose()

	// Read data file.
	buf, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}

	// Rewrite first meta page.
	meta0 := (*meta)(unsafe.Pointer(&buf[pageHeaderSize]))
	meta0.pgid++
	if err := os.WriteFile(path, buf, 0666); err != nil {
		t.Fatal(err)
	}

	// Reopen data file.
	db = btesting.MustOpenDBWithOption(t, path, nil)
	require.Equalf(t, os.Getpagesize(), db.Info().PageSize, "check page size failed")
}

// Ensure that it can read the page size from the second meta page if the first one is invalid.
// The page size is expected to be the given page size in this case.
func TestOpen_ReadPageSize_FromMeta1_Given(t *testing.T) {
	// test page size from 1KB (1024<<0) to 16MB(1024<<14)
	for i := 0; i <= 14; i++ {
		givenPageSize := 1024 << uint(i)
		t.Logf("Testing page size %d", givenPageSize)
		// Create empty database.
		db := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: givenPageSize})
		path := db.Path()
		// Close the database
		db.MustClose()

		// Read data file.
		buf, err := os.ReadFile(path)
		require.NoError(t, err)

		// Rewrite meta pages.
		if i%3 == 0 {
			t.Logf("#%d: Intentionally corrupt the first meta page for pageSize %d", i, givenPageSize)
			meta0 := (*meta)(unsafe.Pointer(&buf[pageHeaderSize]))
			meta0.pgid++
			err = os.WriteFile(path, buf, 0666)
			require.NoError(t, err)
		}

		// Reopen data file.
		db = btesting.MustOpenDBWithOption(t, path, nil)
		require.Equalf(t, givenPageSize, db.Info().PageSize, "check page size failed")
		db.MustClose()
	}
}

// Ensure that opening a database does not increase its size.
// https://github.com/boltdb/bolt/issues/291
func TestOpen_Size(t *testing.T) {
	// Open a data file.
	db := btesting.MustCreateDB(t)

	pagesize := db.Info().PageSize

	// Insert until we get above the minimum 4MB size.
	err := db.Fill([]byte("data"), 1, 10000,
		func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
		func(tx int, k int) []byte { return make([]byte, 1000) },
	)
	if err != nil {
		t.Fatal(err)
	}

	path := db.Path()
	db.MustClose()

	sz := fileSize(path)
	if sz == 0 {
		t.Fatalf("unexpected new file size: %d", sz)
	}

	db.MustReopen()
	if err := db.Update(func(tx *bolt.Tx) error {
		if err := tx.Bucket([]byte("data")).Put([]byte{0}, []byte{0}); err != nil {
			t.Fatal(err)
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
	if err := db.Close(); err != nil {
		t.Fatal(err)
	}
	newSz := fileSize(path)
	if newSz == 0 {
		t.Fatalf("unexpected new file size: %d", newSz)
	}

	// Compare the original size with the new size.
	// db size might increase by a few page sizes due to the new small update.
	if sz < newSz-5*int64(pagesize) {
		t.Fatalf("unexpected file growth: %d => %d", sz, newSz)
	}
}

// Ensure that opening a database beyond the max step size does not increase its size.
// https://github.com/boltdb/bolt/issues/303
func TestOpen_Size_Large(t *testing.T) {
	if testing.Short() {
		t.Skip("short mode")
	}

	// Open a data file.
	db := btesting.MustCreateDB(t)
	path := db.Path()

	pagesize := db.Info().PageSize

	// Insert until we get above the minimum 4MB size.
	var index uint64
	for i := 0; i < 10000; i++ {
		if err := db.Update(func(tx *bolt.Tx) error {
			b, _ := tx.CreateBucketIfNotExists([]byte("data"))
			for j := 0; j < 1000; j++ {
				if err := b.Put(u64tob(index), make([]byte, 50)); err != nil {
					t.Fatal(err)
				}
				index++
			}
			return nil
		}); err != nil {
			t.Fatal(err)
		}
	}

	// Close database and grab the size.
	if err := db.Close(); err != nil {
		t.Fatal(err)
	}
	sz := fileSize(path)
	if sz == 0 {
		t.Fatalf("unexpected new file size: %d", sz)
	} else if sz < (1 << 30) {
		t.Fatalf("expected larger initial size: %d", sz)
	}

	// Reopen database, update, and check size again.
	db0, err := bolt.Open(path, 0600, nil)
	if err != nil {
		t.Fatal(err)
	}
	if err := db0.Update(func(tx *bolt.Tx) error {
		return tx.Bucket([]byte("data")).Put([]byte{0}, []byte{0})
	}); err != nil {
		t.Fatal(err)
	}
	if err := db0.Close(); err != nil {
		t.Fatal(err)
	}

	newSz := fileSize(path)
	if newSz == 0 {
		t.Fatalf("unexpected new file size: %d", newSz)
	}

	// Compare the original size with the new size.
	// db size might increase by a few page sizes due to the new small update.
	if sz < newSz-5*int64(pagesize) {
		t.Fatalf("unexpected file growth: %d => %d", sz, newSz)
	}
}

// Ensure that a re-opened database is consistent.
func TestOpen_Check(t *testing.T) {
	path := tempfile()
	defer os.RemoveAll(path)

	db, err := bolt.Open(path, 0600, nil)
	if err != nil {
		t.Fatal(err)
	}
	if err = db.View(func(tx *bolt.Tx) error { return <-tx.Check() }); err != nil {
		t.Fatal(err)
	}
	if err = db.Close(); err != nil {
		t.Fatal(err)
	}

	db, err = bolt.Open(path, 0600, nil)
	if err != nil {
		t.Fatal(err)
	}
	if err := db.View(func(tx *bolt.Tx) error { return <-tx.Check() }); err != nil {
		t.Fatal(err)
	}
	if err := db.Close(); err != nil {
		t.Fatal(err)
	}
}

// Ensure that write errors to the meta file handler during initialization are returned.
func TestOpen_MetaInitWriteError(t *testing.T) {
	t.Skip("pending")
}

// Ensure that a database that is too small returns an error.
func TestOpen_FileTooSmall(t *testing.T) {
	path := tempfile()
	defer os.RemoveAll(path)

	db, err := bolt.Open(path, 0600, nil)
	if err != nil {
		t.Fatal(err)
	}
	pageSize := int64(db.Info().PageSize)
	if err = db.Close(); err != nil {
		t.Fatal(err)
	}

	// corrupt the database
	if err = os.Truncate(path, pageSize); err != nil {
		t.Fatal(err)
	}

	_, err = bolt.Open(path, 0600, nil)
	if err == nil || !strings.Contains(err.Error(), "file size too small") {
		t.Fatalf("unexpected error: %s", err)
	}
}

// TestDB_Open_InitialMmapSize tests if having InitialMmapSize large enough
// to hold data from concurrent write transaction resolves the issue that
// read transaction blocks the write transaction and causes deadlock.
// This is a very hacky test since the mmap size is not exposed.
func TestDB_Open_InitialMmapSize(t *testing.T) {
	path := tempfile()
	defer os.Remove(path)

	initMmapSize := 1 << 30  // 1GB
	testWriteSize := 1 << 27 // 134MB

	db, err := bolt.Open(path, 0600, &bolt.Options{InitialMmapSize: initMmapSize})
	if err != nil {
		t.Fatal(err)
	}

	// create a long-running read transaction
	// that never gets closed while writing
	rtx, err := db.Begin(false)
	if err != nil {
		t.Fatal(err)
	}

	// create a write transaction
	wtx, err := db.Begin(true)
	if err != nil {
		t.Fatal(err)
	}

	b, err := wtx.CreateBucket([]byte("test"))
	if err != nil {
		t.Fatal(err)
	}

	// and commit a large write
	err = b.Put([]byte("foo"), make([]byte, testWriteSize))
	if err != nil {
		t.Fatal(err)
	}

	done := make(chan error, 1)

	go func() {
		err := wtx.Commit()
		done <- err
	}()

	select {
	case <-time.After(5 * time.Second):
		t.Errorf("unexpected that the reader blocks writer")
	case err := <-done:
		if err != nil {
			t.Fatal(err)
		}
	}

	if err := rtx.Rollback(); err != nil {
		t.Fatal(err)
	}
}

// TestDB_Open_ReadOnly checks a database in read only mode can read but not write.
func TestDB_Open_ReadOnly(t *testing.T) {
	// Create a writable db, write k-v and close it.
	db := btesting.MustCreateDB(t)

	if err := db.Update(func(tx *bolt.Tx) error {
		b, err := tx.CreateBucket([]byte("widgets"))
		if err != nil {
			t.Fatal(err)
		}
		if err := b.Put([]byte("foo"), []byte("bar")); err != nil {
			t.Fatal(err)
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
	if err := db.Close(); err != nil {
		t.Fatal(err)
	}

	f := db.Path()
	o := &bolt.Options{ReadOnly: true}
	readOnlyDB, err := bolt.Open(f, 0600, o)
	if err != nil {
		panic(err)
	}

	if !readOnlyDB.IsReadOnly() {
		t.Fatal("expect db in read only mode")
	}

	// Read from a read-only transaction.
	if err := readOnlyDB.View(func(tx *bolt.Tx) error {
		value := tx.Bucket([]byte("widgets")).Get([]byte("foo"))
		if !bytes.Equal(value, []byte("bar")) {
			t.Fatal("expect value 'bar', got", value)
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}

	// Can't launch read-write transaction.
	if _, err := readOnlyDB.Begin(true); err != berrors.ErrDatabaseReadOnly {
		t.Fatalf("unexpected error: %s", err)
	}

	if err := readOnlyDB.Close(); err != nil {
		t.Fatal(err)
	}
}

func TestDB_Open_ReadOnly_NoCreate(t *testing.T) {
	f := filepath.Join(t.TempDir(), "db")
	_, err := bolt.Open(f, 0600, &bolt.Options{ReadOnly: true})
	require.ErrorIs(t, err, os.ErrNotExist)
}

// TestOpen_BigPage checks the database uses bigger pages when
// changing PageSize.
func TestOpen_BigPage(t *testing.T) {
	pageSize := os.Getpagesize()

	db1 := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize * 2})

	db2 := btesting.MustCreateDBWithOption(t, &bolt.Options{PageSize: pageSize * 4})

	if db1sz, db2sz := fileSize(db1.Path()), fileSize(db2.Path()); db1sz >= db2sz {
		t.Errorf("expected %d < %d", db1sz, db2sz)
	}
}

// TestOpen_RecoverFreeList tests opening the DB with free-list
// write-out after no free list sync will recover the free list
// and write it out.
func TestOpen_RecoverFreeList(t *testing.T) {
	db := btesting.MustCreateDBWithOption(t, &bolt.Options{NoFreelistSync: true})

	// Write some pages.
	tx, err := db.Begin(true)
	if err != nil {
		t.Fatal(err)
	}
	wbuf := make([]byte, 8192)
	for i := 0; i < 100; i++ {
		s := fmt.Sprintf("%d", i)
		b, err := tx.CreateBucket([]byte(s))
		if err != nil {
			t.Fatal(err)
		}
		if err = b.Put([]byte(s), wbuf); err != nil {
			t.Fatal(err)
		}
	}
	if err = tx.Commit(); err != nil {
		t.Fatal(err)
	}

	// Generate free pages.
	if tx, err = db.Begin(true); err != nil {
		t.Fatal(err)
	}
	for i := 0; i < 50; i++ {
		s := fmt.Sprintf("%d", i)
		b := tx.Bucket([]byte(s))
		if b == nil {
			t.Fatal(err)
		}
		if err := b.Delete([]byte(s)); err != nil {
			t.Fatal(err)
		}
	}
	if err := tx.Commit(); err != nil {
		t.Fatal(err)
	}
	db.MustClose()

	// Record freelist count from opening with NoFreelistSync.
	db.MustReopen()
	freepages := db.Stats().FreePageN
	if freepages == 0 {
		t.Fatalf("no free pages on NoFreelistSync reopen")
	}
	db.MustClose()

	// Check free page count is reconstructed when opened with freelist sync.
	db.SetOptions(&bolt.Options{})
	db.MustReopen()
	// One less free page for syncing the free list on open.
	freepages--
	if fp := db.Stats().FreePageN; fp < freepages {
		t.Fatalf("closed with %d free pages, opened with %d", freepages, fp)
	}
}

// Ensure that a database cannot open a transaction when it's not open.
func TestDB_Begin_ErrDatabaseNotOpen(t *testing.T) {
	var db bolt.DB
	if _, err := db.Begin(false); err != berrors.ErrDatabaseNotOpen {
		t.Fatalf("unexpected error: %s", err)
	}
}

// Ensure that a read-write transaction can be retrieved.
func TestDB_BeginRW(t *testing.T) {
	db := btesting.MustCreateDB(t)

	tx, err := db.Begin(true)
	require.NoError(t, err)
	require.NotNil(t, tx, "expected tx")
	defer func() { require.NoError(t, tx.Commit()) }()

	require.True(t, tx.Writable(), "expected writable tx")
	require.Same(t, db.DB, tx.DB())
}

// TestDB_Concurrent_WriteTo checks that issuing WriteTo operations concurrently
// with commits does not produce corrupted db files.
func TestDB_Concurrent_WriteTo(t *testing.T) {
	o := &bolt.Options{NoFreelistSync: false}
	db := btesting.MustCreateDBWithOption(t, o)

	var wg sync.WaitGroup
	wtxs, rtxs := 5, 5
	wg.Add(wtxs * rtxs)
	f := func(tx *bolt.Tx) {
		defer wg.Done()
		f, err := os.CreateTemp("", "bolt-")
		if err != nil {
			panic(err)
		}
		time.Sleep(time.Duration(rand.Intn(20)+1) * time.Millisecond)
		_, err = tx.WriteTo(f)
		if err != nil {
			panic(err)
		}
		err = tx.Rollback()
		if err != nil {
			panic(err)
		}
		f.Close()

		copyOpt := *o
		snap := btesting.MustOpenDBWithOption(t, f.Name(), &copyOpt)
		defer snap.MustClose()
		snap.MustCheck()
	}

	tx1, err := db.Begin(true)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tx1.CreateBucket([]byte("abc")); err != nil {
		t.Fatal(err)
	}
	if err := tx1.Commit(); err != nil {
		t.Fatal(err)
	}

	for i := 0; i < wtxs; i++ {
		tx, err := db.Begin(true)
		if err != nil {
			t.Fatal(err)
		}
		if err := tx.Bucket([]byte("abc")).Put([]byte{0}, []byte{0}); err != nil {
			t.Fatal(err)
		}
		for j := 0; j < rtxs; j++ {
			rtx, rerr := db.Begin(false)
			if rerr != nil {
				t.Fatal(rerr)
			}
			go f(rtx)
		}
		if err := tx.Commit(); err != nil {
			t.Fatal(err)
		}
	}
	wg.Wait()
}

// Ensure that opening a transaction while the DB is closed returns an error.
func TestDB_BeginRW_Closed(t *testing.T) {
	var db bolt.DB
	if _, err := db.Begin(true); err != berrors.ErrDatabaseNotOpen {
		t.Fatalf("unexpected error: %s", err)
	}
}

func TestDB_Close_PendingTx_RW(t *testing.T) { testDB_Close_PendingTx(t, true) }
func TestDB_Close_PendingTx_RO(t *testing.T) { testDB_Close_PendingTx(t, false) }

// Ensure that a database cannot close while transactions are open.
func testDB_Close_PendingTx(t *testing.T, writable bool) {
	db := btesting.MustCreateDB(t)

	// Start transaction.
	tx, err := db.Begin(writable)
	if err != nil {
		t.Fatal(err)
	}

	// Open update in separate goroutine.
	startCh := make(chan struct{}, 1)
	done := make(chan error, 1)
	go func() {
		startCh <- struct{}{}
		err := db.Close()
		done <- err
	}()
	// wait for the above goroutine to get scheduled.
	<-startCh

	// Ensure database hasn't closed.
	time.Sleep(100 * time.Millisecond)
	select {
	case err := <-done:
		if err != nil {
			t.Errorf("error from inside goroutine: %v", err)
		}
		t.Fatal("database closed too early")
	default:
	}

	// Commit/close transaction.
	if writable {
		err = tx.Commit()
	} else {
		err = tx.Rollback()
	}
	if err != nil {
		t.Fatal(err)
	}

	// Ensure database closed now.
	select {
	case err := <-done:
		if err != nil {
			t.Fatalf("error from inside goroutine: %v", err)
		}
	case <-time.After(5 * time.Second):
		t.Fatalf("database did not close")
	}
}

// Ensure a database can provide a transactional block.
func TestDB_Update(t *testing.T) {
	db := btesting.MustCreateDB(t)
	if err := db.Update(func(tx *bolt.Tx) error {
		b, err := tx.CreateBucket([]byte("widgets"))
		if err != nil {
			t.Fatal(err)
		}
		if err := b.Put([]byte("foo"), []byte("bar")); err != nil {
			t.Fatal(err)
		}
		if err := b.Put([]byte("baz"), []byte("bat")); err != nil {
			t.Fatal(err)
		}
		if err := b.Delete([]byte("foo")); err != nil {
			t.Fatal(err)
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
	if err := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket([]byte("widgets"))
		if v := b.Get([]byte("foo")); v != nil {
			t.Fatalf("expected nil value, got: %v", v)
		}
		if v := b.Get([]byte("baz")); !bytes.Equal(v, []byte("bat")) {
			t.Fatalf("unexpected value: %v", v)
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
}

// Ensure a closed database returns an error while running a transaction block
func TestDB_Update_Closed(t *testing.T) {
	var db bolt.DB
	if err := db.Update(func(tx *bolt.Tx) error {
		if _, err := tx.CreateBucket([]byte("widgets")); err != nil {
			t.Fatal(err)
		}
		return nil
	}); err != berrors.ErrDatabaseNotOpen {
		t.Fatalf("unexpected error: %s", err)
	}
}

// Ensure a panic occurs while trying to commit a managed transaction.
func TestDB_Update_ManualCommit(t *testing.T) {
	db := btesting.MustCreateDB(t)

	var panicked bool
	if err := db.Update(func(tx *bolt.Tx) error {
		func() {
			defer func() {
				if r := recover(); r != nil {
					panicked = true
				}
			}()

			if err := tx.Commit(); err != nil {
				t.Fatal(err)
			}
		}()
		return nil
	}); err != nil {
		t.Fatal(err)
	} else if !panicked {
		t.Fatal("expected panic")
	}
}

// Ensure a panic occurs while trying to rollback a managed transaction.
func TestDB_Update_ManualRollback(t *testing.T) {
	db := btesting.MustCreateDB(t)

	var panicked bool
	if err := db.Update(func(tx *bolt.Tx) error {
		func() {
			defer func() {
				if r := recover(); r != nil {
					panicked = true
				}
			}()

			if err := tx.Rollback(); err != nil {
				t.Fatal(err)
			}
		}()
		return nil
	}); err != nil {
		t.Fatal(err)
	} else if !panicked {
		t.Fatal("expected panic")
	}
}

// Ensure a panic occurs while trying to commit a managed transaction.
func TestDB_View_ManualCommit(t *testing.T) {
	db := btesting.MustCreateDB(t)

	var panicked bool
	if err := db.View(func(tx *bolt.Tx) error {
		func() {
			defer func() {
				if r := recover(); r != nil {
					panicked = true
				}
			}()

			if err := tx.Commit(); err != nil {
				t.Fatal(err)
			}
		}()
		return nil
	}); err != nil {
		t.Fatal(err)
	} else if !panicked {
		t.Fatal("expected panic")
	}
}

// Ensure a panic occurs while trying to rollback a managed transaction.
func TestDB_View_ManualRollback(t *testing.T) {
	db := btesting.MustCreateDB(t)

	var panicked bool
	if err := db.View(func(tx *bolt.Tx) error {
		func() {
			defer func() {
				if r := recover(); r != nil {
					panicked = true
				}
			}()

			if err := tx.Rollback(); err != nil {
				t.Fatal(err)
			}
		}()
		return nil
	}); err != nil {
		t.Fatal(err)
	} else if !panicked {
		t.Fatal("expected panic")
	}
}

// Ensure a write transaction that panics does not hold open locks.
func TestDB_Update_Panic(t *testing.T) {
	db := btesting.MustCreateDB(t)

	// Panic during update but recover.
	func() {
		defer func() {
			if r := recover(); r != nil {
				t.Log("recover: update", r)
			}
		}()

		if err := db.Update(func(tx *bolt.Tx) error {
			if _, err := tx.CreateBucket([]byte("widgets")); err != nil {
				t.Fatal(err)
			}
			panic("omg")
		}); err != nil {
			t.Fatal(err)
		}
	}()

	// Verify we can update again.
	if err := db.Update(func(tx *bolt.Tx) error {
		if _, err := tx.CreateBucket([]byte("widgets")); err != nil {
			t.Fatal(err)
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}

	// Verify that our change persisted.
	if err := db.Update(func(tx *bolt.Tx) error {
		if tx.Bucket([]byte("widgets")) == nil {
			t.Fatal("expected bucket")
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
}

// Ensure a database can return an error through a read-only transactional block.
func TestDB_View_Error(t *testing.T) {
	db := btesting.MustCreateDB(t)

	if err := db.View(func(tx *bolt.Tx) error {
		return errors.New("xxx")
	}); err == nil || err.Error() != "xxx" {
		t.Fatalf("unexpected error: %s", err)
	}
}

// Ensure a read transaction that panics does not hold open locks.
func TestDB_View_Panic(t *testing.T) {
	db := btesting.MustCreateDB(t)

	if err := db.Update(func(tx *bolt.Tx) error {
		if _, err := tx.CreateBucket([]byte("widgets")); err != nil {
			t.Fatal(err)
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}

	// Panic during view transaction but recover.
	func() {
		defer func() {
			if r := recover(); r != nil {
				t.Log("recover: view", r)
			}
		}()

		if err := db.View(func(tx *bolt.Tx) error {
			if tx.Bucket([]byte("widgets")) == nil {
				t.Fatal("expected bucket")
			}
			panic("omg")
		}); err != nil {
			t.Fatal(err)
		}
	}()

	// Verify that we can still use read transactions.
	if err := db.View(func(tx *bolt.Tx) error {
		if tx.Bucket([]byte("widgets")) == nil {
			t.Fatal("expected bucket")
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
}

// Ensure that DB stats can be returned.
func TestDB_Stats(t *testing.T) {
	db := btesting.MustCreateDB(t)
	if err := db.Update(func(tx *bolt.Tx) error {
		_, err := tx.CreateBucket([]byte("widgets"))
		return err
	}); err != nil {
		t.Fatal(err)
	}

	stats := db.Stats()
	if stats.TxStats.GetPageCount() != 2 {
		t.Fatalf("unexpected TxStats.PageCount: %d", stats.TxStats.GetPageCount())
	} else if stats.FreePageN != 0 {
		t.Fatalf("unexpected FreePageN != 0: %d", stats.FreePageN)
	} else if stats.PendingPageN != 2 {
		t.Fatalf("unexpected PendingPageN != 2: %d", stats.PendingPageN)
	}
}

// Ensure that database pages are in expected order and type.
func TestDB_Consistency(t *testing.T) {
	db := btesting.MustCreateDB(t)
	if err := db.Update(func(tx *bolt.Tx) error {
		_, err := tx.CreateBucket([]byte("widgets"))
		return err
	}); err != nil {
		t.Fatal(err)
	}

	for i := 0; i < 10; i++ {
		if err := db.Update(func(tx *bolt.Tx) error {
			if err := tx.Bucket([]byte("widgets")).Put([]byte("foo"), []byte("bar")); err != nil {
				t.Fatal(err)
			}
			return nil
		}); err != nil {
			t.Fatal(err)
		}
	}

	if err := db.Update(func(tx *bolt.Tx) error {
		if p, _ := tx.Page(0); p == nil {
			t.Fatal("expected page")
		} else if p.Type != "meta" {
			t.Fatalf("unexpected page type: %s", p.Type)
		}

		if p, _ := tx.Page(1); p == nil {
			t.Fatal("expected page")
		} else if p.Type != "meta" {
			t.Fatalf("unexpected page type: %s", p.Type)
		}

		if p, _ := tx.Page(2); p == nil {
			t.Fatal("expected page")
		} else if p.Type != "free" {
			t.Fatalf("unexpected page type: %s", p.Type)
		}

		if p, _ := tx.Page(3); p == nil {
			t.Fatal("expected page")
		} else if p.Type != "free" {
			t.Fatalf("unexpected page type: %s", p.Type)
		}

		if p, _ := tx.Page(4); p == nil {
			t.Fatal("expected page")
		} else if p.Type != "leaf" {
			t.Fatalf("unexpected page type: %s", p.Type)
		}

		if p, _ := tx.Page(5); p == nil {
			t.Fatal("expected page")
		} else if p.Type != "freelist" {
			t.Fatalf("unexpected page type: %s", p.Type)
		}

		if p, _ := tx.Page(6); p != nil {
			t.Fatal("unexpected page")
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
}

// Ensure that DB stats can be subtracted from one another.
func TestDBStats_Sub(t *testing.T) {
	var a, b bolt.Stats
	a.TxStats.PageCount = 3
	a.FreePageN = 4
	b.TxStats.PageCount = 10
	b.FreePageN = 14
	diff := b.Sub(&a)
	if diff.TxStats.GetPageCount() != 7 {
		t.Fatalf("unexpected TxStats.PageCount: %d", diff.TxStats.GetPageCount())
	}

	// free page stats are copied from the receiver and not subtracted
	if diff.FreePageN != 14 {
		t.Fatalf("unexpected FreePageN: %d", diff.FreePageN)
	}
}

// Ensure two functions can perform updates in a single batch.
func TestDB_Batch(t *testing.T) {
	db := btesting.MustCreateDB(t)

	if err := db.Update(func(tx *bolt.Tx) error {
		if _, err := tx.CreateBucket([]byte("widgets")); err != nil {
			t.Fatal(err)
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}

	// Iterate over multiple updates in separate goroutines.
	n := 2
	ch := make(chan error, n)
	for i := 0; i < n; i++ {
		go func(i int) {
			ch <- db.Batch(func(tx *bolt.Tx) error {
				return tx.Bucket([]byte("widgets")).Put(u64tob(uint64(i)), []byte{})
			})
		}(i)
	}

	// Check all responses to make sure there's no error.
	for i := 0; i < n; i++ {
		if err := <-ch; err != nil {
			t.Fatal(err)
		}
	}

	// Ensure data is correct.
	if err := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket([]byte("widgets"))
		for i := 0; i < n; i++ {
			if v := b.Get(u64tob(uint64(i))); v == nil {
				t.Errorf("key not found: %d", i)
			}
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
}

func TestDB_Batch_Panic(t *testing.T) {
	db := btesting.MustCreateDB(t)

	var sentinel int
	var bork = &sentinel
	var problem interface{}
	var err error

	// Execute a function inside a batch that panics.
	func() {
		defer func() {
			if p := recover(); p != nil {
				problem = p
			}
		}()
		err = db.Batch(func(tx *bolt.Tx) error {
			panic(bork)
		})
	}()

	// Verify there is no error.
	if g, e := err, error(nil); g != e {
		t.Fatalf("wrong error: %v != %v", g, e)
	}
	// Verify the panic was captured.
	if g, e := problem, bork; g != e {
		t.Fatalf("wrong error: %v != %v", g, e)
	}
}

func TestDB_BatchFull(t *testing.T) {
	db := btesting.MustCreateDB(t)
	if err := db.Update(func(tx *bolt.Tx) error {
		_, err := tx.CreateBucket([]byte("widgets"))
		return err
	}); err != nil {
		t.Fatal(err)
	}

	const size = 3
	// buffered so we never leak goroutines
	ch := make(chan error, size)
	put := func(i int) {
		ch <- db.Batch(func(tx *bolt.Tx) error {
			return tx.Bucket([]byte("widgets")).Put(u64tob(uint64(i)), []byte{})
		})
	}

	db.MaxBatchSize = size
	// high enough to never trigger here
	db.MaxBatchDelay = 1 * time.Hour

	go put(1)
	go put(2)

	// Give the batch a chance to exhibit bugs.
	time.Sleep(10 * time.Millisecond)

	// not triggered yet
	select {
	case <-ch:
		t.Fatalf("batch triggered too early")
	default:
	}

	go put(3)

	// Check all responses to make sure there's no error.
	for i := 0; i < size; i++ {
		if err := <-ch; err != nil {
			t.Fatal(err)
		}
	}

	// Ensure data is correct.
	if err := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket([]byte("widgets"))
		for i := 1; i <= size; i++ {
			if v := b.Get(u64tob(uint64(i))); v == nil {
				t.Errorf("key not found: %d", i)
			}
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
}

func TestDB_BatchTime(t *testing.T) {
	db := btesting.MustCreateDB(t)
	if err := db.Update(func(tx *bolt.Tx) error {
		_, err := tx.CreateBucket([]byte("widgets"))
		return err
	}); err != nil {
		t.Fatal(err)
	}

	const size = 1
	// buffered so we never leak goroutines
	ch := make(chan error, size)
	put := func(i int) {
		ch <- db.Batch(func(tx *bolt.Tx) error {
			return tx.Bucket([]byte("widgets")).Put(u64tob(uint64(i)), []byte{})
		})
	}

	db.MaxBatchSize = 1000
	db.MaxBatchDelay = 0

	go put(1)

	// Batch must trigger by time alone.

	// Check all responses to make sure there's no error.
	for i := 0; i < size; i++ {
		if err := <-ch; err != nil {
			t.Fatal(err)
		}
	}

	// Ensure data is correct.
	if err := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket([]byte("widgets"))
		for i := 1; i <= size; i++ {
			if v := b.Get(u64tob(uint64(i))); v == nil {
				t.Errorf("key not found: %d", i)
			}
		}
		return nil
	}); err != nil {
		t.Fatal(err)
	}
}

// TestDBUnmap verifes that `dataref`, `data` and `datasz` must be reset
// to zero values respectively after unmapping the db.
func TestDBUnmap(t *testing.T) {
	db := btesting.MustCreateDB(t)

	require.NoError(t, db.DB.Close())

	// Ignore the following error:
	// Error: copylocks: call of reflect.ValueOf copies lock value: go.etcd.io/bbolt.DB contains sync.Once contains sync.Mutex (govet)
	//nolint:govet
	v := reflect.ValueOf(*db.DB)
	dataref := v.FieldByName("dataref")
	data := v.FieldByName("data")
	datasz := v.FieldByName("datasz")
	assert.True(t, dataref.IsNil())
	assert.True(t, data.IsNil())
	assert.True(t, datasz.IsZero())

	// Set db.DB to nil to prevent MustCheck from panicking.
	db.DB = nil
}

func ExampleDB_Update() {
	// Open the database.
	db, err := bolt.Open(tempfile(), 0600, nil)
	if err != nil {
		log.Fatal(err)
	}
	defer os.Remove(db.Path())

	// Execute several commands within a read-write transaction.
	if err := db.Update(func(tx *bolt.Tx) error {
		b, err := tx.CreateBucket([]byte("widgets"))
		if err != nil {
			return err
		}
		if err := b.Put([]byte("foo"), []byte("bar")); err != nil {
			return err
		}
		return nil
	}); err != nil {
		log.Fatal(err)
	}

	// Read the value back from a separate read-only transaction.
	if err := db.View(func(tx *bolt.Tx) error {
		value := tx.Bucket([]byte("widgets")).Get([]byte("foo"))
		fmt.Printf("The value of 'foo' is: %s\n", value)
		return nil
	}); err != nil {
		log.Fatal(err)
	}

	// Close database to release the file lock.
	if err := db.Close(); err != nil {
		log.Fatal(err)
	}

	// Output:
	// The value of 'foo' is: bar
}

func ExampleDB_View() {
	// Open the database.
	db, err := bolt.Open(tempfile(), 0600, nil)
	if err != nil {
		log.Fatal(err)
	}
	defer os.Remove(db.Path())

	// Insert data into a bucket.
	if err := db.Update(func(tx *bolt.Tx) error {
		b, err := tx.CreateBucket([]byte("people"))
		if err != nil {
			return err
		}
		if err := b.Put([]byte("john"), []byte("doe")); err != nil {
			return err
		}
		if err := b.Put([]byte("susy"), []byte("que")); err != nil {
			return err
		}
		return nil
	}); err != nil {
		log.Fatal(err)
	}

	// Access data from within a read-only transactional block.
	if err := db.View(func(tx *bolt.Tx) error {
		v := tx.Bucket([]byte("people")).Get([]byte("john"))
		fmt.Printf("John's last name is %s.\n", v)
		return nil
	}); err != nil {
		log.Fatal(err)
	}

	// Close database to release the file lock.
	if err := db.Close(); err != nil {
		log.Fatal(err)
	}

	// Output:
	// John's last name is doe.
}

func ExampleDB_Begin() {
	// Open the database.
	db, err := bolt.Open(tempfile(), 0600, nil)
	if err != nil {
		log.Fatal(err)
	}
	defer os.Remove(db.Path())

	// Create a bucket using a read-write transaction.
	if err = db.Update(func(tx *bolt.Tx) error {
		_, err := tx.CreateBucket([]byte("widgets"))
		return err
	}); err != nil {
		log.Fatal(err)
	}

	// Create several keys in a transaction.
	tx, err := db.Begin(true)
	if err != nil {
		log.Fatal(err)
	}
	b := tx.Bucket([]byte("widgets"))
	if err = b.Put([]byte("john"), []byte("blue")); err != nil {
		log.Fatal(err)
	}
	if err = b.Put([]byte("abby"), []byte("red")); err != nil {
		log.Fatal(err)
	}
	if err = b.Put([]byte("zephyr"), []byte("purple")); err != nil {
		log.Fatal(err)
	}
	if err = tx.Commit(); err != nil {
		log.Fatal(err)
	}

	// Iterate over the values in sorted key order.
	tx, err = db.Begin(false)
	if err != nil {
		log.Fatal(err)
	}
	c := tx.Bucket([]byte("widgets")).Cursor()
	for k, v := c.First(); k != nil; k, v = c.Next() {
		fmt.Printf("%s likes %s\n", k, v)
	}

	if err = tx.Rollback(); err != nil {
		log.Fatal(err)
	}

	if err = db.Close(); err != nil {
		log.Fatal(err)
	}

	// Output:
	// abby likes red
	// john likes blue
	// zephyr likes purple
}

func BenchmarkDBBatchAutomatic(b *testing.B) {
	db := btesting.MustCreateDB(b)

	if err := db.Update(func(tx *bolt.Tx) error {
		_, err := tx.CreateBucket([]byte("bench"))
		return err
	}); err != nil {
		b.Fatal(err)
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		start := make(chan struct{})
		var wg sync.WaitGroup

		for round := 0; round < 1000; round++ {
			wg.Add(1)

			go func(id uint32) {
				defer wg.Done()
				<-start

				h := fnv.New32a()
				buf := make([]byte, 4)
				binary.LittleEndian.PutUint32(buf, id)
				_, _ = h.Write(buf[:])
				k := h.Sum(nil)
				insert := func(tx *bolt.Tx) error {
					b := tx.Bucket([]byte("bench"))
					return b.Put(k, []byte("filler"))
				}
				if err := db.Batch(insert); err != nil {
					b.Error(err)
					return
				}
			}(uint32(round))
		}
		close(start)
		wg.Wait()
	}

	b.StopTimer()
	validateBatchBench(b, db)
}

func BenchmarkDBBatchSingle(b *testing.B) {
	db := btesting.MustCreateDB(b)
	if err := db.Update(func(tx *bolt.Tx) error {
		_, err := tx.CreateBucket([]byte("bench"))
		return err
	}); err != nil {
		b.Fatal(err)
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		start := make(chan struct{})
		var wg sync.WaitGroup

		for round := 0; round < 1000; round++ {
			wg.Add(1)
			go func(id uint32) {
				defer wg.Done()
				<-start

				h := fnv.New32a()
				buf := make([]byte, 4)
				binary.LittleEndian.PutUint32(buf, id)
				_, _ = h.Write(buf[:])
				k := h.Sum(nil)
				insert := func(tx *bolt.Tx) error {
					b := tx.Bucket([]byte("bench"))
					return b.Put(k, []byte("filler"))
				}
				if err := db.Update(insert); err != nil {
					b.Error(err)
					return
				}
			}(uint32(round))
		}
		close(start)
		wg.Wait()
	}

	b.StopTimer()
	validateBatchBench(b, db)
}

func BenchmarkDBBatchManual10x100(b *testing.B) {
	db := btesting.MustCreateDB(b)
	if err := db.Update(func(tx *bolt.Tx) error {
		_, err := tx.CreateBucket([]byte("bench"))
		return err
	}); err != nil {
		b.Fatal(err)
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		start := make(chan struct{})
		var wg sync.WaitGroup
		errCh := make(chan error, 10)

		for major := 0; major < 10; major++ {
			wg.Add(1)
			go func(id uint32) {
				defer wg.Done()
				<-start

				insert100 := func(tx *bolt.Tx) error {
					h := fnv.New32a()
					buf := make([]byte, 4)
					for minor := uint32(0); minor < 100; minor++ {
						binary.LittleEndian.PutUint32(buf, uint32(id*100+minor))
						h.Reset()
						_, _ = h.Write(buf[:])
						k := h.Sum(nil)
						b := tx.Bucket([]byte("bench"))
						if err := b.Put(k, []byte("filler")); err != nil {
							return err
						}
					}
					return nil
				}
				err := db.Update(insert100)
				errCh <- err
			}(uint32(major))
		}
		close(start)
		wg.Wait()
		close(errCh)
		for err := range errCh {
			if err != nil {
				b.Fatal(err)
			}
		}
	}

	b.StopTimer()
	validateBatchBench(b, db)
}

func validateBatchBench(b *testing.B, db *btesting.DB) {
	var rollback = errors.New("sentinel error to cause rollback")
	validate := func(tx *bolt.Tx) error {
		bucket := tx.Bucket([]byte("bench"))
		h := fnv.New32a()
		buf := make([]byte, 4)
		for id := uint32(0); id < 1000; id++ {
			binary.LittleEndian.PutUint32(buf, id)
			h.Reset()
			_, _ = h.Write(buf[:])
			k := h.Sum(nil)
			v := bucket.Get(k)
			if v == nil {
				b.Errorf("not found id=%d key=%x", id, k)
				continue
			}
			if g, e := v, []byte("filler"); !bytes.Equal(g, e) {
				b.Errorf("bad value for id=%d key=%x: %s != %q", id, k, g, e)
			}
			if err := bucket.Delete(k); err != nil {
				return err
			}
		}
		// should be empty now
		c := bucket.Cursor()
		for k, v := c.First(); k != nil; k, v = c.Next() {
			b.Errorf("unexpected key: %x = %q", k, v)
		}
		return rollback
	}
	if err := db.Update(validate); err != nil && err != rollback {
		b.Error(err)
	}
}

// tempfile returns a temporary file path.
func tempfile() string {
	f, err := os.CreateTemp("", "bolt-")
	if err != nil {
		panic(err)
	}
	if err := f.Close(); err != nil {
		panic(err)
	}
	if err := os.Remove(f.Name()); err != nil {
		panic(err)
	}
	return f.Name()
}

func trunc(b []byte, length int) []byte {
	if length < len(b) {
		return b[:length]
	}
	return b
}

func fileSize(path string) int64 {
	fi, err := os.Stat(path)
	if err != nil {
		return 0
	}
	return fi.Size()
}

// u64tob converts a uint64 into an 8-byte slice.
func u64tob(v uint64) []byte {
	b := make([]byte, 8)
	binary.BigEndian.PutUint64(b, v)
	return b
}
