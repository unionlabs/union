package db

import (
	"bytes"
	"crypto/rand"
	"encoding/binary"
	"fmt"
	"path/filepath"
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
)

func mockDBWithStuff(t *testing.T) DB {
	db := NewMemDB()
	// Under "key" prefix
	require.NoError(t, db.Set(bz("key"), bz("value")))
	require.NoError(t, db.Set(bz("key1"), bz("value1")))
	require.NoError(t, db.Set(bz("key2"), bz("value2")))
	require.NoError(t, db.Set(bz("key3"), bz("value3")))
	require.NoError(t, db.Set(bz("something"), bz("else")))
	require.NoError(t, db.Set(bz("k"), bz("val")))
	require.NoError(t, db.Set(bz("ke"), bz("valu")))
	require.NoError(t, db.Set(bz("kee"), bz("valuu")))
	return db
}

func taskKey(i, k int) []byte {
	return []byte(fmt.Sprintf("task-%d-key-%d", i, k))
}

func randomValue() []byte {
	b := make([]byte, 16)
	_, err := rand.Read(b)
	if err != nil {
	    panic(fmt.Sprintf("random value generation failed: %v", err))
	}
	return b
}

func TestGolevelDB(t *testing.T) {
	path := filepath.Join(t.TempDir(), "goleveldb")

	db, err := NewGoLevelDB(path, "", nil)
	require.NoError(t, err)

	Run(t, db)
}

/* We don't seem to test badger anywhere.
func TestWithBadgerDB(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "badgerdb")

	db, err := NewBadgerDB(path, "")
	require.NoError(t, err)

	t.Run("BadgerDB", func(t *testing.T) { Run(t, db) })
}
*/

func TestWithMemDB(t *testing.T) {
	db := NewMemDB()

	t.Run("MemDB", func(t *testing.T) { Run(t, db) })
}

// Run generates concurrent reads and writes to db so the race detector can
// verify concurrent operations are properly synchronized.
// The contents of db are garbage after Run returns.
func Run(t *testing.T, db DB) {
	t.Helper()

	const numWorkers = 10
	const numKeys = 64

	var wg sync.WaitGroup
	for i := 0; i < numWorkers; i++ {
		wg.Add(1)
		i := i
		go func() {
			defer wg.Done()

			// Insert a bunch of keys with random data.
			for k := 1; k <= numKeys; k++ {
				key := taskKey(i, k) // say, "task-<i>-key-<k>"
				value := randomValue()
				if err := db.Set(key, value); err != nil {
					t.Errorf("Task %d: db.Set(%q=%q) failed: %v",
						i, string(key), string(value), err)
				}
			}

			// Iterate over the database to make sure our keys are there.
			it, err := db.Iterator(nil, nil)
			if err != nil {
				t.Errorf("Iterator[%d]: %v", i, err)
				return
			}
			found := make(map[string][]byte)
			mine := []byte(fmt.Sprintf("task-%d-", i))
			for {
				if key := it.Key(); bytes.HasPrefix(key, mine) {
					found[string(key)] = it.Value()
				}
				it.Next()
				if !it.Valid() {
					break
				}
			}
			if err := it.Error(); err != nil {
				t.Errorf("Iterator[%d] reported error: %v", i, err)
			}
			if err := it.Close(); err != nil {
				t.Errorf("Close iterator[%d]: %v", i, err)
			}
			if len(found) != numKeys {
				t.Errorf("Task %d: found %d keys, wanted %d", i, len(found), numKeys)
			}

			// Delete all the keys we inserted.
			for key := range mine {
				bs := make([]byte, 4)
				binary.LittleEndian.PutUint32(bs, uint32(key))
				if err := db.Delete(bs); err != nil {
					t.Errorf("Delete %q: %v", key, err)
				}
			}
		}()
	}
	wg.Wait()
}

func TestPrefixDBSimple(t *testing.T) {
	db := mockDBWithStuff(t)
	pdb := NewPrefixDB(db, bz("key"))

	checkValue(t, pdb, bz("key"), nil)
	checkValue(t, pdb, bz("key1"), nil)
	checkValue(t, pdb, bz("1"), bz("value1"))
	checkValue(t, pdb, bz("key2"), nil)
	checkValue(t, pdb, bz("2"), bz("value2"))
	checkValue(t, pdb, bz("key3"), nil)
	checkValue(t, pdb, bz("3"), bz("value3"))
	checkValue(t, pdb, bz("something"), nil)
	checkValue(t, pdb, bz("k"), nil)
	checkValue(t, pdb, bz("ke"), nil)
	checkValue(t, pdb, bz("kee"), nil)
}

func TestPrefixDBIterator1(t *testing.T) {
	db := mockDBWithStuff(t)
	pdb := NewPrefixDB(db, bz("key"))

	itr, err := pdb.Iterator(nil, nil)
	require.NoError(t, err)
	checkDomain(t, itr, nil, nil)
	checkItem(t, itr, bz("1"), bz("value1"))
	checkNext(t, itr, true)
	checkItem(t, itr, bz("2"), bz("value2"))
	checkNext(t, itr, true)
	checkItem(t, itr, bz("3"), bz("value3"))
	checkNext(t, itr, false)
	checkInvalid(t, itr)
	itr.Close()
}

func TestPrefixDBReverseIterator1(t *testing.T) {
	db := mockDBWithStuff(t)
	pdb := NewPrefixDB(db, bz("key"))

	itr, err := pdb.ReverseIterator(nil, nil)
	require.NoError(t, err)
	checkDomain(t, itr, nil, nil)
	checkItem(t, itr, bz("3"), bz("value3"))
	checkNext(t, itr, true)
	checkItem(t, itr, bz("2"), bz("value2"))
	checkNext(t, itr, true)
	checkItem(t, itr, bz("1"), bz("value1"))
	checkNext(t, itr, false)
	checkInvalid(t, itr)
	itr.Close()
}

func TestPrefixDBReverseIterator5(t *testing.T) {
	db := mockDBWithStuff(t)
	pdb := NewPrefixDB(db, bz("key"))

	itr, err := pdb.ReverseIterator(bz("1"), nil)
	require.NoError(t, err)
	checkDomain(t, itr, bz("1"), nil)
	checkItem(t, itr, bz("3"), bz("value3"))
	checkNext(t, itr, true)
	checkItem(t, itr, bz("2"), bz("value2"))
	checkNext(t, itr, true)
	checkItem(t, itr, bz("1"), bz("value1"))
	checkNext(t, itr, false)
	checkInvalid(t, itr)
	itr.Close()
}

func TestPrefixDBReverseIterator6(t *testing.T) {
	db := mockDBWithStuff(t)
	pdb := NewPrefixDB(db, bz("key"))

	itr, err := pdb.ReverseIterator(bz("2"), nil)
	require.NoError(t, err)
	checkDomain(t, itr, bz("2"), nil)
	checkItem(t, itr, bz("3"), bz("value3"))
	checkNext(t, itr, true)
	checkItem(t, itr, bz("2"), bz("value2"))
	checkNext(t, itr, false)
	checkInvalid(t, itr)
	itr.Close()
}

func TestPrefixDBReverseIterator7(t *testing.T) {
	db := mockDBWithStuff(t)
	pdb := NewPrefixDB(db, bz("key"))

	itr, err := pdb.ReverseIterator(nil, bz("2"))
	require.NoError(t, err)
	checkDomain(t, itr, nil, bz("2"))
	checkItem(t, itr, bz("1"), bz("value1"))
	checkNext(t, itr, false)
	checkInvalid(t, itr)
	itr.Close()
}
