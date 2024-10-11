package levigo

import (
	"bytes"
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"testing"
	"time"
)

func init() {
	rand.Seed(int64(time.Now().Nanosecond()))
}

// This testcase is a port of leveldb's c_test.c.
func TestC(t *testing.T) {
	if GetLevelDBMajorVersion() <= 0 {
		t.Errorf("Major version cannot be less than zero")
	}

	dbname := tempDir(t)
	defer deleteDBDirectory(t, dbname)
	env := NewDefaultEnv()
	cache := NewLRUCache(1 << 20)

	options := NewOptions()
	// options.SetComparator(cmp)
	options.SetErrorIfExists(true)
	options.SetCache(cache)
	options.SetEnv(env)
	options.SetInfoLog(nil)
	options.SetWriteBufferSize(1 << 20)
	options.SetParanoidChecks(true)
	options.SetMaxOpenFiles(10)
	options.SetBlockSize(1024)
	options.SetBlockRestartInterval(8)
	options.SetCompression(NoCompression)

	roptions := NewReadOptions()
	roptions.SetVerifyChecksums(true)
	roptions.SetFillCache(false)

	woptions := NewWriteOptions()
	woptions.SetSync(true)

	_ = DestroyDatabase(dbname, options)

	db, err := Open(dbname, options)
	if err == nil {
		t.Errorf("Open on missing db should have failed")
	}

	options.SetCreateIfMissing(true)
	db, err = Open(dbname, options)
	if err != nil {
		t.Fatalf("Open failed: %v", err)
	}

	putKey := []byte("foo")
	putValue := []byte("hello")
	err = db.Put(woptions, putKey, putValue)
	if err != nil {
		t.Errorf("Put failed: %v", err)
	}

	CheckGet(t, "after Put", db, roptions, putKey, putValue)

	wb := NewWriteBatch()
	wb.Put([]byte("foo"), []byte("a"))
	wb.Clear()
	wb.Put([]byte("bar"), []byte("b"))
	wb.Put([]byte("box"), []byte("c"))
	wb.Delete([]byte("bar"))
	err = db.Write(woptions, wb)
	if err != nil {
		t.Errorf("Write batch failed: %v", err)
	}
	CheckGet(t, "after WriteBatch", db, roptions, []byte("foo"), []byte("hello"))
	CheckGet(t, "after WriteBatch", db, roptions, []byte("bar"), nil)
	CheckGet(t, "after WriteBatch", db, roptions, []byte("box"), []byte("c"))
	// TODO: WriteBatch iteration isn't easy. Suffers same problems as
	// Comparator.
	// wbiter := &TestWBIter{t: t}
	// wb.Iterate(wbiter)
	// if wbiter.pos != 3 {
	// 	t.Errorf("After Iterate, on the wrong pos: %d", wbiter.pos)
	// }
	wb.Close()

	iter := db.NewIterator(roptions)
	if iter.Valid() {
		t.Errorf("Read iterator should not be valid, yet")
	}
	iter.SeekToFirst()
	if !iter.Valid() {
		t.Errorf("Read iterator should be valid after seeking to first record")
	}
	CheckIter(t, iter, []byte("box"), []byte("c"))
	iter.Next()
	CheckIter(t, iter, []byte("foo"), []byte("hello"))
	iter.Prev()
	CheckIter(t, iter, []byte("box"), []byte("c"))
	iter.Prev()
	if iter.Valid() {
		t.Errorf("Read iterator should not be valid after go back past the first record")
	}
	iter.SeekToLast()
	CheckIter(t, iter, []byte("foo"), []byte("hello"))
	iter.Seek([]byte("b"))
	CheckIter(t, iter, []byte("box"), []byte("c"))
	if iter.GetError() != nil {
		t.Errorf("Read iterator has an error we didn't expect: %v", iter.GetError())
	}
	iter.Close()

	// approximate sizes
	n := 20000
	for i := 0; i < n; i++ {
		keybuf := []byte(fmt.Sprintf("k%020d", i))
		valbuf := []byte(fmt.Sprintf("v%020d", i))
		err := db.Put(woptions, keybuf, valbuf)
		if err != nil {
			t.Errorf("Put error in approximate size test: %v", err)
		}
	}

	ranges := []Range{
		{[]byte("a"), []byte("k00000000000000010000")},
		{[]byte("k00000000000000010000"), []byte("z")},
	}
	sizes := db.GetApproximateSizes(ranges)
	if len(sizes) == 2 {
		if sizes[0] <= 0 {
			t.Errorf("First size range was %d", sizes[0])
		}
		if sizes[1] <= 0 {
			t.Errorf("Second size range was %d", sizes[1])
		}
	} else {
		t.Errorf("Expected 2 approx. sizes back, got %d", len(sizes))
	}

	// property
	prop := db.PropertyValue("nosuchprop")
	if prop != "" {
		t.Errorf("property nosuchprop should not have a value")
	}
	prop = db.PropertyValue("leveldb.stats")
	if prop == "" {
		t.Errorf("property leveldb.stats should have a value")
	}

	// snapshot
	snap := db.NewSnapshot()
	err = db.Delete(woptions, []byte("foo"))
	if err != nil {
		t.Errorf("Delete during snapshot test errored: %v", err)
	}
	roptions.SetSnapshot(snap)
	CheckGet(t, "from snapshot", db, roptions, []byte("foo"), []byte("hello"))
	roptions.SetSnapshot(nil)
	CheckGet(t, "from snapshot", db, roptions, []byte("foo"), nil)
	db.ReleaseSnapshot(snap)

	// repair
	db.Close()
	options.SetCreateIfMissing(false)
	options.SetErrorIfExists(false)
	err = RepairDatabase(dbname, options)
	if err != nil {
		t.Errorf("Repairing db failed: %v", err)
	}
	db, err = Open(dbname, options)
	if err != nil {
		t.Errorf("Unable to open repaired db: %v", err)
	}
	CheckGet(t, "repair", db, roptions, []byte("foo"), nil)
	CheckGet(t, "repair", db, roptions, []byte("bar"), nil)
	CheckGet(t, "repair", db, roptions, []byte("box"), []byte("c"))
	options.SetCreateIfMissing(true)
	options.SetErrorIfExists(true)

	// filter
	policy := NewBloomFilter(10)
	db.Close()
	DestroyDatabase(dbname, options)
	options.SetFilterPolicy(policy)
	db, err = Open(dbname, options)
	if err != nil {
		t.Fatalf("Unable to recreate db for filter tests: %v", err)
	}
	err = db.Put(woptions, []byte("foo"), []byte("foovalue"))
	if err != nil {
		t.Errorf("Unable to put 'foo' with filter: %v", err)
	}
	err = db.Put(woptions, []byte("bar"), []byte("barvalue"))
	if err != nil {
		t.Errorf("Unable to put 'bar' with filter: %v", err)
	}
	db.CompactRange(Range{nil, nil})
	CheckGet(t, "filter", db, roptions, []byte("foo"), []byte("foovalue"))
	CheckGet(t, "filter", db, roptions, []byte("bar"), []byte("barvalue"))
	options.SetFilterPolicy(nil)
	policy.Close()

	// cleanup
	db.Close()
	options.Close()
	roptions.Close()
	woptions.Close()
	cache.Close()
	// DestroyComparator(cmp)
	env.Close()
}

func TestNilSlicesInDb(t *testing.T) {
	dbname := tempDir(t)
	defer deleteDBDirectory(t, dbname)
	options := NewOptions()
	options.SetErrorIfExists(true)
	options.SetCreateIfMissing(true)
	ro := NewReadOptions()
	_ = DestroyDatabase(dbname, options)
	db, err := Open(dbname, options)
	if err != nil {
		t.Fatalf("Database could not be opened: %v", err)
	}
	defer db.Close()
	val, err := db.Get(ro, []byte("missing"))
	if err != nil {
		t.Errorf("Get failed: %v", err)
	}
	if val != nil {
		t.Errorf("A key not in the db should return nil, not %v", val)
	}
	wo := NewWriteOptions()
	db.Put(wo, nil, []byte("love"))
	val, err = db.Get(ro, nil)
	if !bytes.Equal([]byte("love"), val) {
		t.Errorf("Get should see the nil key: %v", val)
	}
	val, err = db.Get(ro, []byte{})
	if !bytes.Equal([]byte("love"), val) {
		t.Errorf("Get shouldn't distinguish between nil key and empty slice key: %v", val)
	}

	err = db.Put(wo, []byte("nilvalue"), nil)
	if err != nil {
		t.Errorf("nil value Put errored: %v", err)
	}
	// Compare with the []byte("missing") case. We expect Get to return a
	// []byte{} here, but expect a nil returned there.
	CheckGet(t, "nil value Put", db, ro, []byte("nilvalue"), []byte{})

	err = db.Put(wo, []byte("emptyvalue"), []byte{})
	if err != nil {
		t.Errorf("empty value Put errored: %v", err)
	}
	CheckGet(t, "empty value Put", db, ro, []byte("emptyvalue"), []byte{})

	err = db.Delete(wo, nil)
	if err != nil {
		t.Errorf("nil key Delete errored: %v", err)
	}
	err = db.Delete(wo, []byte{})
	if err != nil {
		t.Errorf("empty slice key Delete errored: %v", err)
	}

}

func TestIterationValidityLimits(t *testing.T) {
	dbname := tempDir(t)
	defer deleteDBDirectory(t, dbname)
	options := NewOptions()
	options.SetErrorIfExists(true)
	options.SetCreateIfMissing(true)
	ro := NewReadOptions()
	wo := NewWriteOptions()
	_ = DestroyDatabase(dbname, options)
	db, err := Open(dbname, options)
	if err != nil {
		t.Fatalf("Database could not be opened: %v", err)
	}
	defer db.Close()
	db.Put(wo, []byte("bat"), []byte("somedata"))
	db.Put(wo, []byte("done"), []byte("somedata"))
	it := db.NewIterator(ro)
	defer it.Close()
	if it.Valid() {
		t.Errorf("new Iterator was valid")
	}
	it.Seek([]byte("bat"))
	if !it.Valid() {
		t.Errorf("Seek to %#v failed.", []byte("bat"))
	}
	if !bytes.Equal([]byte("bat"), it.Key()) {
		t.Errorf("did not seek to []byte(\"bat\")")
	}
	key := it.Key()
	it.Next()
	if bytes.Equal(key, it.Key()) {
		t.Errorf("key should be a copy of last key")
	}
	it.Next()
	if it.Valid() {
		t.Errorf("iterating off the db should result in an invalid iterator")
	}
	err = it.GetError()
	if err != nil {
		t.Errorf("should not have seen an error on an invalid iterator")
	}
	it.Seek([]byte("bat"))
	if !it.Valid() {
		t.Errorf("Iterator should be valid again")
	}
}

func CheckGet(t *testing.T, where string, db *DB, roptions *ReadOptions, key, expected []byte) {
	getValue, err := db.Get(roptions, key)

	if err != nil {
		t.Errorf("%s, Get failed: %v", where, err)
	}
	if !bytes.Equal(getValue, expected) {
		t.Errorf("%s, expected Get value %v, got %v", where, expected, getValue)
	}
}

func WBIterCheckEqual(t *testing.T, where string, which string, pos int, expected, given []byte) {
	if !bytes.Equal(expected, given) {
		t.Errorf("%s at pos %d, %s expected: %v, got: %v", where, pos, which, expected, given)
	}
}

func CheckIter(t *testing.T, it *Iterator, key, value []byte) {
	if !bytes.Equal(key, it.Key()) {
		t.Errorf("Iterator: expected key %v, got %v", key, it.Key())
	}
	if !bytes.Equal(value, it.Value()) {
		t.Errorf("Iterator: expected value %v, got %v", value, it.Value())
	}
}

func deleteDBDirectory(t *testing.T, dirPath string) {
	err := os.RemoveAll(dirPath)
	if err != nil {
		t.Errorf("Unable to remove database directory: %s", dirPath)
	}
}

func tempDir(t *testing.T) string {
	bottom := fmt.Sprintf("levigo-test-%d", rand.Int())
	path := filepath.Join(os.TempDir(), bottom)
	deleteDBDirectory(t, path)
	return path
}
