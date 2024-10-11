package db

import (
	"bytes"
	"encoding/binary"
	"math/rand"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ----------------------------------------
// Helper functions.

func checkValue(t *testing.T, db DB, key []byte, valueWanted []byte) {
	t.Helper()
	valueGot, err := db.Get(key)
	assert.NoError(t, err)
	assert.Equal(t, valueWanted, valueGot)
}

func checkValid(t *testing.T, itr Iterator, expected bool) {
	t.Helper()
	valid := itr.Valid()
	require.Equal(t, expected, valid)
}

func checkNext(t *testing.T, itr Iterator, expected bool) {
	t.Helper()
	itr.Next()
	// assert.NoError(t, err) TODO: look at fixing this
	valid := itr.Valid()
	require.Equal(t, expected, valid)
}

func checkNextPanics(t *testing.T, itr Iterator) {
	t.Helper()
	assert.Panics(t, func() { itr.Next() }, "checkNextPanics expected an error but didn't")
}

func checkDomain(t *testing.T, itr Iterator, start, end []byte) {
	t.Helper()
	ds, de := itr.Domain()
	assert.Equal(t, start, ds, "checkDomain domain start incorrect")
	assert.Equal(t, end, de, "checkDomain domain end incorrect")
}

func checkItem(t *testing.T, itr Iterator, key []byte, value []byte) {
	t.Helper()
	v := itr.Value()

	k := itr.Key()

	assert.Exactly(t, key, k)
	assert.Exactly(t, value, v)
}

func checkInvalid(t *testing.T, itr Iterator) {
	t.Helper()
	checkValid(t, itr, false)
	checkKeyPanics(t, itr)
	checkValuePanics(t, itr)
	checkNextPanics(t, itr)
}

func checkKeyPanics(t *testing.T, itr Iterator) {
	t.Helper()
	assert.Panics(t, func() { itr.Key() }, "checkKeyPanics expected panic but didn't")
}

func checkValuePanics(t *testing.T, itr Iterator) {
	t.Helper()

	msg := "checkValuePanics expected panic but didn't"
	assert.Panics(t, func() { itr.Value() }, msg)
}

func newTempDB(t *testing.T, backend BackendType) (db DB, dbDir string) {
	t.Helper()
	dirname, err := os.MkdirTemp("", "db_common_test")
	require.NoError(t, err)
	db, err = NewDB("testdb", backend, dirname)
	require.NoError(t, err)
	return db, dirname
}

func benchmarkRangeScans(b *testing.B, db DB, dbSize int64) {
	b.Helper()
	b.StopTimer()

	rangeSize := int64(10000)
	if dbSize < rangeSize {
		b.Errorf("db size %v cannot be less than range size %v", dbSize, rangeSize)
	}

	for i := int64(0); i < dbSize; i++ {
		int64bytes := int642Bytes(i)
		err := db.Set(int64bytes, int64bytes)
		if err != nil {
			// require.NoError() is very expensive (according to profiler), so check manually
			b.Fatal(b, err)
		}
	}
	b.StartTimer()

	for i := 0; i < b.N; i++ {
		start := rand.Int63n(dbSize - rangeSize)
		end := start + rangeSize
		iter, err := db.Iterator(int642Bytes(start), int642Bytes(end))
		require.NoError(b, err)
		count := 0
		for ; iter.Valid(); iter.Next() {
			count++
		}
		err = iter.Close()
		require.NoError(b, err)
		require.EqualValues(b, rangeSize, count)
	}
}

func benchmarkRandomReadsWrites(b *testing.B, db DB) {
	b.Helper()
	b.StopTimer()

	// create dummy data
	const numItems = int64(1000000)
	internal := map[int64]int64{}
	for i := 0; i < int(numItems); i++ {
		internal[int64(i)] = int64(0)
	}

	b.StartTimer()

	for i := 0; i < b.N; i++ {
		// Write something
		{
			idx := rand.Int63n(numItems)
			internal[idx]++
			val := internal[idx]
			idxBytes := int642Bytes(idx)
			valBytes := int642Bytes(val)
			err := db.Set(idxBytes, valBytes)
			if err != nil {
				// require.NoError() is very expensive (according to profiler), so check manually
				b.Fatal(b, err)
			}
		}

		// Read something
		{
			idx := rand.Int63n(numItems)
			valExp := internal[idx]
			idxBytes := int642Bytes(idx)
			valBytes, err := db.Get(idxBytes)
			if err != nil {
				// require.NoError() is very expensive (according to profiler), so check manually
				b.Fatal(b, err)
			}
			if valExp == 0 {
				if !bytes.Equal(valBytes, nil) {
					b.Errorf("Expected %v for %v, got %X", nil, idx, valBytes)
					break
				}
			} else {
				if len(valBytes) != 8 {
					b.Errorf("Expected length 8 for %v, got %X", idx, valBytes)
					break
				}
				valGot := bytes2Int64(valBytes)
				if valExp != valGot {
					b.Errorf("Expected %v for %v, got %v", valExp, idx, valGot)
					break
				}
			}
		}
	}
}

func int642Bytes(i int64) []byte {
	buf := make([]byte, 8)
	binary.BigEndian.PutUint64(buf, uint64(i))
	return buf
}

func bytes2Int64(buf []byte) int64 {
	return int64(binary.BigEndian.Uint64(buf))
}
