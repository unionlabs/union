package grocksdb

import (
	"bytes"
	"runtime"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestComparatorWithTS(t *testing.T) {
	t.Parallel()

	db, opts := newTestDBAndOpts(t, func(opts *Options) {
		comp := newComparatorWithTimeStamp(
			"rev",
			func(a, b []byte) int {
				return bytes.Compare(a, b) * -1
			},
		)
		opts.SetComparator(comp)
	})
	defer func() {
		db.Close()
		opts.Destroy()
	}()

	runtime.GC()

	// insert keys
	givenKeys := [][]byte{[]byte("key1"), []byte("key2"), []byte("key3")}
	givenTimes := [][]byte{marshalTimestamp(1), marshalTimestamp(2), marshalTimestamp(3)}

	wo := NewDefaultWriteOptions()
	for i, k := range givenKeys {
		require.Nil(t, db.PutWithTS(wo, k, givenTimes[i], []byte("val")))
		runtime.GC()
	}

	// create a iterator to collect the keys
	ro := NewDefaultReadOptions()
	ro.SetTimestamp(marshalTimestamp(4))
	iter := db.NewIterator(ro)
	defer iter.Close()

	// we seek to the last key and iterate in reverse order
	// to match given keys
	var actualKeys, actualTimes [][]byte
	for iter.SeekToLast(); iter.Valid(); iter.Prev() {
		key := make([]byte, 4)
		ts := make([]byte, timestampSize)
		copy(key, iter.Key().Data())
		copy(ts, iter.Timestamp().Data())
		actualKeys = append(actualKeys, key)
		actualTimes = append(actualTimes, ts)
		runtime.GC()
	}
	require.Nil(t, iter.Err())

	// ensure that the order is correct
	require.EqualValues(t, actualKeys, givenKeys)
	require.EqualValues(t, actualTimes, givenTimes)
}
