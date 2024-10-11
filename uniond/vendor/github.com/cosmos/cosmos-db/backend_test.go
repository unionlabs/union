package db

import (
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Register a test backend for PrefixDB as well, with some unrelated junk data
func init() {
	registerDBCreator("prefixdb", func(name, dir string, opts Options) (DB, error) {
		mdb := NewMemDB()
		mdb.Set([]byte("a"), []byte{1})    //nolint:errcheck
		mdb.Set([]byte("b"), []byte{2})    //nolint:errcheck
		mdb.Set([]byte("t"), []byte{20})   //nolint:errcheck
		mdb.Set([]byte("test"), []byte{0}) //nolint:errcheck
		mdb.Set([]byte("u"), []byte{21})   //nolint:errcheck
		mdb.Set([]byte("z"), []byte{26})   //nolint:errcheck
		return NewPrefixDB(mdb, []byte("test/")), nil
	}, false)
}

func cleanupDBDir(dir, name string) {
	err := os.RemoveAll(filepath.Join(dir, name) + DBFileSuffix)
	if err != nil {
		panic(err)
	}
}

func testBackendGetSetDelete(t *testing.T, backend BackendType) {
	// Default
	dirname, err := os.MkdirTemp("", fmt.Sprintf("test_backend_%s_", backend))
	require.Nil(t, err)
	db, err := NewDB("testdb", backend, dirname)
	require.NoError(t, err)
	defer cleanupDBDir(dirname, "testdb")

	// A nonexistent key should return nil.
	value, err := db.Get([]byte("a"))
	require.NoError(t, err)
	require.Nil(t, value)

	ok, err := db.Has([]byte("a"))
	require.NoError(t, err)
	require.False(t, ok)

	// Set and get a value.
	err = db.Set([]byte("a"), []byte{0x01})
	require.NoError(t, err)

	ok, err = db.Has([]byte("a"))
	require.NoError(t, err)
	require.True(t, ok)

	value, err = db.Get([]byte("a"))
	require.NoError(t, err)
	require.Equal(t, []byte{0x01}, value)

	err = db.SetSync([]byte("b"), []byte{0x02})
	require.NoError(t, err)

	value, err = db.Get([]byte("b"))
	require.NoError(t, err)
	require.Equal(t, []byte{0x02}, value)

	// Deleting a non-existent value is fine.
	err = db.Delete([]byte("x"))
	require.NoError(t, err)

	err = db.DeleteSync([]byte("x"))
	require.NoError(t, err)

	// Delete a value.
	err = db.Delete([]byte("a"))
	require.NoError(t, err)

	value, err = db.Get([]byte("a"))
	require.NoError(t, err)
	require.Nil(t, value)

	err = db.DeleteSync([]byte("b"))
	require.NoError(t, err)

	value, err = db.Get([]byte("b"))
	require.NoError(t, err)
	require.Nil(t, value)

	// Setting, getting, and deleting an empty key should error.
	_, err = db.Get([]byte{})
	require.Equal(t, errKeyEmpty, err)
	_, err = db.Get(nil)
	require.Equal(t, errKeyEmpty, err)

	_, err = db.Has([]byte{})
	require.Equal(t, errKeyEmpty, err)
	_, err = db.Has(nil)
	require.Equal(t, errKeyEmpty, err)

	err = db.Set([]byte{}, []byte{0x01})
	require.Equal(t, errKeyEmpty, err)
	err = db.Set(nil, []byte{0x01})
	require.Equal(t, errKeyEmpty, err)
	err = db.SetSync([]byte{}, []byte{0x01})
	require.Equal(t, errKeyEmpty, err)
	err = db.SetSync(nil, []byte{0x01})
	require.Equal(t, errKeyEmpty, err)

	err = db.Delete([]byte{})
	require.Equal(t, errKeyEmpty, err)
	err = db.Delete(nil)
	require.Equal(t, errKeyEmpty, err)
	err = db.DeleteSync([]byte{})
	require.Equal(t, errKeyEmpty, err)
	err = db.DeleteSync(nil)
	require.Equal(t, errKeyEmpty, err)

	// Setting a nil value should error, but an empty value is fine.
	err = db.Set([]byte("x"), nil)
	require.Equal(t, errValueNil, err)
	err = db.SetSync([]byte("x"), nil)
	require.Equal(t, errValueNil, err)

	err = db.Set([]byte("x"), []byte{})
	require.NoError(t, err)
	err = db.SetSync([]byte("x"), []byte{})
	require.NoError(t, err)
	value, err = db.Get([]byte("x"))
	require.NoError(t, err)
	require.Equal(t, []byte{}, value)
}

func TestBackendsGetSetDelete(t *testing.T) {
	for dbType := range backends {
		t.Run(string(dbType), func(t *testing.T) {
			testBackendGetSetDelete(t, dbType)
		})
	}
}

func TestGoLevelDBBackend(t *testing.T) {
	name := fmt.Sprintf("test_%x", randStr(12))
	db, err := NewDB(name, GoLevelDBBackend, "")
	require.NoError(t, err)
	defer cleanupDBDir("", name)

	_, ok := db.(*GoLevelDB)
	assert.True(t, ok)
}

func TestDBIterator(t *testing.T) {
	for dbType := range backends {
		t.Run(string(dbType), func(t *testing.T) {
			testDBIterator(t, dbType)
		})
	}
}

func testDBIterator(t *testing.T, backend BackendType) {
	name := fmt.Sprintf("test_%x", randStr(12))
	dir := os.TempDir()
	db, err := NewDB(name, backend, dir)
	require.NoError(t, err)
	defer cleanupDBDir(dir, name)

	for i := 0; i < 10; i++ {
		if i != 6 { // but skip 6.
			err := db.Set(int642Bytes(int64(i)), []byte{})
			require.NoError(t, err)
		}
	}

	// Blank iterator keys should error
	_, err = db.Iterator([]byte{}, nil)
	require.Equal(t, errKeyEmpty, err)
	_, err = db.Iterator(nil, []byte{})
	require.Equal(t, errKeyEmpty, err)
	_, err = db.ReverseIterator([]byte{}, nil)
	require.Equal(t, errKeyEmpty, err)
	_, err = db.ReverseIterator(nil, []byte{})
	require.Equal(t, errKeyEmpty, err)

	itr, err := db.Iterator(nil, nil)
	require.NoError(t, err)
	verifyIterator(t, itr, []int64{0, 1, 2, 3, 4, 5, 7, 8, 9}, "forward iterator")

	ritr, err := db.ReverseIterator(nil, nil)
	require.NoError(t, err)
	verifyIterator(t, ritr, []int64{9, 8, 7, 5, 4, 3, 2, 1, 0}, "reverse iterator")

	itr, err = db.Iterator(nil, int642Bytes(0))
	require.NoError(t, err)
	verifyIterator(t, itr, []int64(nil), "forward iterator to 0")

	ritr, err = db.ReverseIterator(int642Bytes(10), nil)
	require.NoError(t, err)
	verifyIterator(t, ritr, []int64(nil), "reverse iterator from 10 (ex)")

	itr, err = db.Iterator(int642Bytes(0), nil)
	require.NoError(t, err)
	verifyIterator(t, itr, []int64{0, 1, 2, 3, 4, 5, 7, 8, 9}, "forward iterator from 0")

	itr, err = db.Iterator(int642Bytes(1), nil)
	require.NoError(t, err)
	verifyIterator(t, itr, []int64{1, 2, 3, 4, 5, 7, 8, 9}, "forward iterator from 1")

	ritr, err = db.ReverseIterator(nil, int642Bytes(10))
	require.NoError(t, err)
	verifyIterator(t, ritr,
		[]int64{9, 8, 7, 5, 4, 3, 2, 1, 0}, "reverse iterator from 10 (ex)")

	ritr, err = db.ReverseIterator(nil, int642Bytes(9))
	require.NoError(t, err)
	verifyIterator(t, ritr,
		[]int64{8, 7, 5, 4, 3, 2, 1, 0}, "reverse iterator from 9 (ex)")

	ritr, err = db.ReverseIterator(nil, int642Bytes(8))
	require.NoError(t, err)
	verifyIterator(t, ritr,
		[]int64{7, 5, 4, 3, 2, 1, 0}, "reverse iterator from 8 (ex)")

	itr, err = db.Iterator(int642Bytes(5), int642Bytes(6))
	require.NoError(t, err)
	verifyIterator(t, itr, []int64{5}, "forward iterator from 5 to 6")

	itr, err = db.Iterator(int642Bytes(5), int642Bytes(7))
	require.NoError(t, err)
	verifyIterator(t, itr, []int64{5}, "forward iterator from 5 to 7")

	itr, err = db.Iterator(int642Bytes(5), int642Bytes(8))
	require.NoError(t, err)
	verifyIterator(t, itr, []int64{5, 7}, "forward iterator from 5 to 8")

	itr, err = db.Iterator(int642Bytes(6), int642Bytes(7))
	require.NoError(t, err)
	verifyIterator(t, itr, []int64(nil), "forward iterator from 6 to 7")

	itr, err = db.Iterator(int642Bytes(6), int642Bytes(8))
	require.NoError(t, err)
	verifyIterator(t, itr, []int64{7}, "forward iterator from 6 to 8")

	itr, err = db.Iterator(int642Bytes(7), int642Bytes(8))
	require.NoError(t, err)
	verifyIterator(t, itr, []int64{7}, "forward iterator from 7 to 8")

	ritr, err = db.ReverseIterator(int642Bytes(4), int642Bytes(5))
	require.NoError(t, err)
	verifyIterator(t, ritr, []int64{4}, "reverse iterator from 5 (ex) to 4")

	ritr, err = db.ReverseIterator(int642Bytes(4), int642Bytes(6))
	require.NoError(t, err)
	verifyIterator(t, ritr,
		[]int64{5, 4}, "reverse iterator from 6 (ex) to 4")

	ritr, err = db.ReverseIterator(int642Bytes(4), int642Bytes(7))
	require.NoError(t, err)
	verifyIterator(t, ritr,
		[]int64{5, 4}, "reverse iterator from 7 (ex) to 4")

	ritr, err = db.ReverseIterator(int642Bytes(5), int642Bytes(6))
	require.NoError(t, err)
	verifyIterator(t, ritr, []int64{5}, "reverse iterator from 6 (ex) to 5")

	ritr, err = db.ReverseIterator(int642Bytes(5), int642Bytes(7))
	require.NoError(t, err)
	verifyIterator(t, ritr, []int64{5}, "reverse iterator from 7 (ex) to 5")

	ritr, err = db.ReverseIterator(int642Bytes(6), int642Bytes(7))
	require.NoError(t, err)
	verifyIterator(t, ritr,
		[]int64(nil), "reverse iterator from 7 (ex) to 6")

	ritr, err = db.ReverseIterator(int642Bytes(10), nil)
	require.NoError(t, err)
	verifyIterator(t, ritr, []int64(nil), "reverse iterator to 10")

	ritr, err = db.ReverseIterator(int642Bytes(6), nil)
	require.NoError(t, err)
	verifyIterator(t, ritr, []int64{9, 8, 7}, "reverse iterator to 6")

	ritr, err = db.ReverseIterator(int642Bytes(5), nil)
	require.NoError(t, err)
	verifyIterator(t, ritr, []int64{9, 8, 7, 5}, "reverse iterator to 5")

	ritr, err = db.ReverseIterator(int642Bytes(8), int642Bytes(9))
	require.NoError(t, err)
	verifyIterator(t, ritr, []int64{8}, "reverse iterator from 9 (ex) to 8")

	ritr, err = db.ReverseIterator(int642Bytes(2), int642Bytes(4))
	require.NoError(t, err)
	verifyIterator(t, ritr,
		[]int64{3, 2}, "reverse iterator from 4 (ex) to 2")

	ritr, err = db.ReverseIterator(int642Bytes(4), int642Bytes(2))
	require.NoError(t, err)
	verifyIterator(t, ritr,
		[]int64(nil), "reverse iterator from 2 (ex) to 4")

	// Ensure that the iterators don't panic with an empty database.
	dir2, err := os.MkdirTemp("", "tm-db-test")
	require.NoError(t, err)
	db2, err := NewDB(name, backend, dir2)
	require.NoError(t, err)
	defer cleanupDBDir(dir2, name)

	itr, err = db2.Iterator(nil, nil)
	require.NoError(t, err)
	verifyIterator(t, itr, nil, "forward iterator with empty db")

	ritr, err = db2.ReverseIterator(nil, nil)
	require.NoError(t, err)
	verifyIterator(t, ritr, nil, "reverse iterator with empty db")
}

func verifyIterator(t *testing.T, itr Iterator, expected []int64, msg string) {
	var list []int64
	for itr.Valid() {
		key := itr.Key()
		list = append(list, bytes2Int64(key))
		itr.Next()
	}
	assert.Equal(t, expected, list, msg)
}

func TestDBBatchGetByteSize(t *testing.T) {
	for dbType := range backends {
		t.Run(fmt.Sprintf("%v", dbType), func(t *testing.T) {
			testDBBatchGetByteSize(t, dbType)
		})
	}
}

func testDBBatchGetByteSize(t *testing.T, backend BackendType) {
	name := fmt.Sprintf("test_%x", randStr(12))
	dir := os.TempDir()
	db, err := NewDB(name, backend, dir)
	require.NoError(t, err)
	defer cleanupDBDir(dir, name)

	// create a new batch
	batch := db.NewBatch()
	batchSize, err := batch.GetByteSize()
	require.NoError(t, err)
	// size of newly created batch should be 0 or negligible because of the metadata in the batch,
	// for example peppble's batchHeaderLen is 12 so
	// peppble's batch size will always be equal or greater than 12 even for empty batch
	require.LessOrEqual(t, batchSize, 32)

	totalSizeOfKeyAndValue := 0
	// set 100 random keys and values
	for i := 0; i < 100; i++ {
		keySize := rand.Intn(32) + 1   //nolint:gosec
		valueSize := rand.Intn(32) + 1 //nolint:gosec
		totalSizeOfKeyAndValue += keySize + valueSize
		require.NoError(t, batch.Set([]byte(randStr(keySize)), []byte(randStr(valueSize))))
	}

	batchSize, err = batch.GetByteSize()
	require.NoError(t, err)
	// because we set a lot of keys and values with considerable size,
	// ratio of batchSize / totalSizeOfKeyAndValue should be roughly 1
	require.Equal(t, 1, batchSize/totalSizeOfKeyAndValue)

	err = batch.Write()
	require.NoError(t, err)

	_, err = batch.GetByteSize()
	// calling GetByteSize on a written batch should error
	require.Error(t, err)
}

func TestDBBatchOperations(t *testing.T) {
	for dbType := range backends {
		t.Run(fmt.Sprintf("%v", dbType), func(t *testing.T) {
			testDBBatchOperations(t, dbType)
		})
	}
}

func testDBBatchOperations(t *testing.T, backend BackendType) {
	name := fmt.Sprintf("test_%x", randStr(12))
	dir := os.TempDir()
	db, err := NewDB(name, backend, dir)
	require.NoError(t, err)
	defer cleanupDBDir(dir, name)

	// create a new batch, and some items - they should not be visible until we write
	batch := db.NewBatch()
	require.NoError(t, batch.Set([]byte("a"), []byte{1}))
	require.NoError(t, batch.Set([]byte("b"), []byte{2}))
	require.NoError(t, batch.Set([]byte("c"), []byte{3}))
	assertKeyValues(t, db, map[string][]byte{})

	err = batch.Write()
	require.NoError(t, err)
	assertKeyValues(t, db, map[string][]byte{"a": {1}, "b": {2}, "c": {3}})

	// trying to modify or rewrite a written batch should error, but closing it should work
	require.Error(t, batch.Set([]byte("a"), []byte{9}))
	require.Error(t, batch.Delete([]byte("a")))
	require.Error(t, batch.Write())
	require.Error(t, batch.WriteSync())
	require.NoError(t, batch.Close())

	// batches should write changes in order
	batch = db.NewBatch()
	require.NoError(t, batch.Delete([]byte("a")))
	require.NoError(t, batch.Set([]byte("a"), []byte{1}))
	require.NoError(t, batch.Set([]byte("b"), []byte{1}))
	require.NoError(t, batch.Set([]byte("b"), []byte{2}))
	require.NoError(t, batch.Set([]byte("c"), []byte{3}))
	require.NoError(t, batch.Delete([]byte("c")))
	require.NoError(t, batch.Write())
	require.NoError(t, batch.Close())
	assertKeyValues(t, db, map[string][]byte{"a": {1}, "b": {2}})

	// empty and nil keys, as well as nil values, should be disallowed
	batch = db.NewBatch()
	err = batch.Set([]byte{}, []byte{0x01})
	require.Equal(t, errKeyEmpty, err)
	err = batch.Set(nil, []byte{0x01})
	require.Equal(t, errKeyEmpty, err)
	err = batch.Set([]byte("a"), nil)
	require.Equal(t, errValueNil, err)

	err = batch.Delete([]byte{})
	require.Equal(t, errKeyEmpty, err)
	err = batch.Delete(nil)
	require.Equal(t, errKeyEmpty, err)

	err = batch.Close()
	require.NoError(t, err)

	// it should be possible to write an empty batch
	batch = db.NewBatch()
	err = batch.Write()
	require.NoError(t, err)
	assertKeyValues(t, db, map[string][]byte{"a": {1}, "b": {2}})

	// it should be possible to close an empty batch, and to re-close a closed batch
	batch = db.NewBatch()
	batch.Close()
	batch.Close()

	// all other operations on a closed batch should error
	require.Error(t, batch.Set([]byte("a"), []byte{9}))
	require.Error(t, batch.Delete([]byte("a")))
	require.Error(t, batch.Write())
	require.Error(t, batch.WriteSync())
}

func assertKeyValues(t *testing.T, db DB, expect map[string][]byte) {
	iter, err := db.Iterator(nil, nil)
	require.NoError(t, err)
	defer iter.Close()

	actual := make(map[string][]byte)
	for ; iter.Valid(); iter.Next() {
		require.NoError(t, iter.Error())
		actual[string(iter.Key())] = iter.Value()
	}

	assert.Equal(t, expect, actual)
}
