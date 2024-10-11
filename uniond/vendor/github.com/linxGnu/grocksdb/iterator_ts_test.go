package grocksdb

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestIteratorWithTS(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, func(opts *Options) {
		opts.SetComparator(newDefaultComparatorWithTS())
	})
	defer db.Close()

	// insert keys
	givenKeys := [][]byte{[]byte("key1"), []byte("key2"), []byte("key3")}
	givenTimes := [][]byte{marshalTimestamp(1), marshalTimestamp(2), marshalTimestamp(3)}
	wo := NewDefaultWriteOptions()
	for i, k := range givenKeys {
		require.Nil(t, db.PutWithTS(wo, k, givenTimes[i], []byte("val")))
	}

	ro := NewDefaultReadOptions()
	defer ro.Destroy()

	ro.SetTimestamp(marshalTimestamp(4))
	iter := db.NewIterator(ro)
	defer iter.Close()
	var actualKeys, actualTimes [][]byte
	for iter.SeekToFirst(); iter.Valid(); iter.Next() {
		key := make([]byte, 4)
		copy(key, iter.Key().Data())
		ts := make([]byte, timestampSize)
		copy(ts, iter.Timestamp().Data())
		actualKeys = append(actualKeys, key)
		actualTimes = append(actualTimes, ts)
	}
	require.Nil(t, iter.Err())
	require.EqualValues(t, actualKeys, givenKeys)
	require.EqualValues(t, actualTimes, givenTimes)

	// Should Only read key1
	ro.SetTimestamp(marshalTimestamp(1))
	iter = db.NewIterator(ro)
	defer iter.Close()

	actualKeys = actualKeys[:0]
	actualTimes = actualTimes[:0]
	givenKeys = [][]byte{[]byte("key1")}
	givenTimes = [][]byte{marshalTimestamp(1)}

	for iter.SeekToFirst(); iter.Valid(); iter.Next() {
		key := make([]byte, 4)
		copy(key, iter.Key().Data())
		ts := make([]byte, timestampSize)
		copy(ts, iter.Timestamp().Data())
		actualKeys = append(actualKeys, key)
		actualTimes = append(actualTimes, ts)
	}
	require.Nil(t, iter.Err())
	require.EqualValues(t, actualKeys, givenKeys)
	require.EqualValues(t, actualTimes, givenTimes)

	// Should read none
	ro.SetTimestamp(marshalTimestamp(0))
	iter = db.NewIterator(ro)
	defer iter.Close()

	actualKeys = actualKeys[:0]
	actualTimes = actualTimes[:0]
	givenKeys = givenKeys[:0]
	givenTimes = givenTimes[:0]

	for iter.SeekToFirst(); iter.Valid(); iter.Next() {
		key := make([]byte, 4)
		copy(key, iter.Key().Data())
		ts := make([]byte, timestampSize)
		copy(ts, iter.Timestamp().Data())
		actualKeys = append(actualKeys, key)
		actualTimes = append(actualTimes, ts)
	}
	require.Nil(t, iter.Err())
	require.EqualValues(t, actualKeys, givenKeys)
	require.EqualValues(t, actualTimes, givenTimes)
}

func TestIteratorWriteManyThenIterWithTS(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, func(opts *Options) {
		opts.SetComparator(newDefaultComparatorWithTS())
	})
	defer db.Close()

	numKey := 10_000

	ts := marshalTimestamp(1)

	// insert keys
	wo := NewDefaultWriteOptions()
	for i := 0; i < numKey; i++ {
		require.Nil(t, db.PutWithTS(wo, []byte(fmt.Sprintf("key_%d", i)), ts, []byte("val")))
	}

	for attempt := 0; attempt < 400; attempt++ {
		ro := NewDefaultReadOptions()
		ro.SetTimestamp(ts)
		ro.SetIterateUpperBound([]byte("keya"))

		iter, count := db.NewIterator(ro), 0
		for iter.SeekToFirst(); iter.Valid(); iter.Next() {
			count++
		}

		require.NoError(t, iter.Err())
		require.EqualValues(t, numKey, count)

		ro.Destroy()
		iter.Close()
	}
}

func TestIteratorCFWithTS(t *testing.T) {
	t.Parallel()

	db, cfs, cleanup := newTestDBMultiCF(t, []string{"default", "c1", "c2", "c3"}, func(opts *Options) {
		opts.SetComparator(newDefaultComparatorWithTS())
	})
	defer cleanup()

	ts4 := marshalTimestamp(4)

	// insert keys
	givenKeys := [][]byte{[]byte("key1"), []byte("key2"), []byte("key3")}
	givenTimes := [][]byte{marshalTimestamp(1), marshalTimestamp(2), marshalTimestamp(3)}
	wo := NewDefaultWriteOptions()
	for j, k := range givenKeys {
		for i := range cfs {
			require.Nil(t, db.PutCFWithTS(wo, cfs[i], k, givenTimes[j], []byte("val")))
		}
	}

	{
		ro := NewDefaultReadOptions()
		ro.SetTimestamp(ts4)
		iter := db.NewIteratorCF(ro, cfs[0])
		defer iter.Close()
		var actualKeys, actualTimes [][]byte
		for iter.SeekToFirst(); iter.Valid(); iter.Next() {
			key := make([]byte, 4)
			ts := make([]byte, timestampSize)
			copy(key, iter.Key().Data())
			copy(ts, iter.Timestamp().Data())
			actualKeys = append(actualKeys, key)
			actualTimes = append(actualTimes, ts)
		}
		require.Nil(t, iter.Err())
		require.EqualValues(t, actualKeys, givenKeys)
		require.EqualValues(t, actualTimes, givenTimes)
	}

	{
		ro := NewDefaultReadOptions()
		ro.SetTimestamp(ts4)
		iters, err := db.NewIterators(ro, cfs)
		require.Nil(t, err)
		require.EqualValues(t, len(iters), 4)
		defer func() {
			for i := range iters {
				iters[i].Close()
			}
		}()

		for _, iter := range iters {
			var actualKeys, actualTimes [][]byte
			for iter.SeekToFirst(); iter.Valid(); iter.Next() {
				key := make([]byte, 4)
				ts := make([]byte, timestampSize)
				copy(key, iter.Key().Data())
				copy(ts, iter.Timestamp().Data())
				actualKeys = append(actualKeys, key)
				actualTimes = append(actualTimes, ts)
			}
			require.Nil(t, iter.Err())
			require.EqualValues(t, actualKeys, givenKeys)
			require.EqualValues(t, actualTimes, givenTimes)
		}
	}
}

func TestIteratorRangeWithTS(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, func(opts *Options) {
		opts.SetComparator(newDefaultComparatorWithTS())
	})
	defer db.Close()

	givenKey1 := []byte("key1")
	givenKey2 := []byte("key2")
	givenKey3 := []byte("key3")

	givenTs1 := marshalTimestamp(1)
	givenTs2 := marshalTimestamp(2)
	givenTs3 := marshalTimestamp(3)
	givenTs4 := marshalTimestamp(4)
	givenTs5 := marshalTimestamp(5)

	// insert keys
	wo := NewDefaultWriteOptions()
	require.Nil(t, db.PutWithTS(wo, givenKey1, givenTs1, []byte("val1")))
	require.Nil(t, db.PutWithTS(wo, givenKey1, givenTs2, []byte("val2")))
	require.Nil(t, db.PutWithTS(wo, givenKey2, givenTs3, []byte("value1")))
	require.Nil(t, db.DeleteWithTS(wo, givenKey1, givenTs4))
	require.Nil(t, db.DeleteWithTS(wo, givenKey2, givenTs4))
	require.Nil(t, db.PutWithTS(wo, givenKey3, givenTs4, []byte("value2")))

	ro := NewDefaultReadOptions()
	defer ro.Destroy()

	// ts5 should read only key3
	ro.SetTimestamp(givenTs5)
	iter := db.NewIterator(ro)
	defer iter.Close()
	var actualKeys, actualTimes [][]byte
	for iter.SeekToFirst(); iter.Valid(); iter.Next() {
		key := make([]byte, 4)
		copy(key, iter.Key().Data())
		ts := make([]byte, timestampSize)
		copy(ts, iter.Timestamp().Data())
		actualKeys = append(actualKeys, key)
		actualTimes = append(actualTimes, ts)
	}
	require.Nil(t, iter.Err())
	require.EqualValues(t, actualKeys, [][]byte{givenKey3})
	require.EqualValues(t, actualTimes, [][]byte{givenTs4})

	// range from ts1 to ts5, should have 6 keys
	actualKeys = actualKeys[:0]
	actualTimes = actualTimes[:0]

	givenKeys := [][]byte{
		givenKey1,
		givenKey1,
		givenKey1,
		givenKey2,
		givenKey2,
		givenKey3,
	}

	givenTimes := [][]byte{
		givenTs4,
		givenTs2,
		givenTs1,
		givenTs4,
		givenTs3,
		givenTs4,
	}

	ro.SetIterStartTimestamp(givenTs1)
	ro.SetTimestamp(givenTs5)
	iter = db.NewIterator(ro)
	defer iter.Close()
	for iter.SeekToFirst(); iter.Valid(); iter.Next() {
		k := iter.Key().Data()
		internalKey := make([]byte, len(k))
		copy(internalKey, k)

		key, ts := extractFromInteralKey(internalKey)
		actualKeys = append(actualKeys, key)
		actualTimes = append(actualTimes, ts)
	}
	require.Nil(t, iter.Err())
	require.EqualValues(t, actualKeys, givenKeys)
	require.EqualValues(t, actualTimes, givenTimes)
}
