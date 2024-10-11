package grocksdb

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestIterator(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	// insert keys
	givenKeys := [][]byte{[]byte("key1"), []byte("key2"), []byte("key3")}
	wo := NewDefaultWriteOptions()
	for _, k := range givenKeys {
		require.Nil(t, db.Put(wo, k, []byte("val")))
	}

	ro := NewDefaultReadOptions()
	iter := db.NewIterator(ro)
	defer iter.Close()
	var actualKeys [][]byte
	for iter.SeekToFirst(); iter.Valid(); iter.Next() {
		key := make([]byte, 4)
		copy(key, iter.Key().Data())
		actualKeys = append(actualKeys, key)
	}
	require.Nil(t, iter.Err())
	require.EqualValues(t, actualKeys, givenKeys)
}

func TestIteratorWriteManyThenIter(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	numKey := 10_000

	// insert keys
	wo := NewDefaultWriteOptions()
	for i := 0; i < numKey; i++ {
		require.Nil(t, db.Put(wo, []byte(fmt.Sprintf("key_%d", i)), []byte("val")))
	}

	for attempt := 0; attempt < 400; attempt++ {
		ro := NewDefaultReadOptions()
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

func TestIteratorCF(t *testing.T) {
	t.Parallel()

	db, cfs, cleanup := newTestDBMultiCF(t, []string{"default", "c1", "c2", "c3"}, nil)
	defer cleanup()

	// insert keys
	givenKeys := [][]byte{[]byte("key1"), []byte("key2"), []byte("key3")}
	wo := NewDefaultWriteOptions()
	for _, k := range givenKeys {
		for i := range cfs {
			require.Nil(t, db.PutCF(wo, cfs[i], k, []byte("val")))
		}
	}

	{
		ro := NewDefaultReadOptions()
		iter := db.NewIteratorCF(ro, cfs[0])
		defer iter.Close()
		var actualKeys [][]byte
		for iter.SeekToFirst(); iter.Valid(); iter.Next() {
			key := make([]byte, 4)
			copy(key, iter.Key().Data())
			actualKeys = append(actualKeys, key)
		}
		require.Nil(t, iter.Err())
		require.EqualValues(t, actualKeys, givenKeys)
	}

	{
		ro := NewDefaultReadOptions()
		iters, err := db.NewIterators(ro, cfs)
		require.Nil(t, err)
		require.EqualValues(t, len(iters), 4)
		defer func() {
			for i := range iters {
				iters[i].Close()
			}
		}()

		for _, iter := range iters {
			var actualKeys [][]byte
			for iter.SeekToFirst(); iter.Valid(); iter.Next() {
				key := make([]byte, 4)
				copy(key, iter.Key().Data())
				actualKeys = append(actualKeys, key)
			}
			require.Nil(t, iter.Err())
			require.EqualValues(t, actualKeys, givenKeys)
		}
	}
}
