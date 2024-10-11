package internal

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/store/types"
)

func TestGetSetDelete(t *testing.T) {
	db := NewBTree()

	// A nonexistent key should return nil.
	value := db.Get([]byte("a"))
	require.Nil(t, value)

	// Set and get a value.
	db.Set([]byte("a"), []byte{0x01})
	db.Set([]byte("b"), []byte{0x02})
	value = db.Get([]byte("a"))
	require.Equal(t, []byte{0x01}, value)

	value = db.Get([]byte("b"))
	require.Equal(t, []byte{0x02}, value)

	// Deleting a non-existent value is fine.
	db.Delete([]byte("x"))

	// Delete a value.
	db.Delete([]byte("a"))

	value = db.Get([]byte("a"))
	require.Nil(t, value)

	db.Delete([]byte("b"))

	value = db.Get([]byte("b"))
	require.Nil(t, value)
}

func TestDBIterator(t *testing.T) {
	db := NewBTree()

	for i := 0; i < 10; i++ {
		if i != 6 { // but skip 6.
			db.Set(int642Bytes(int64(i)), []byte{})
		}
	}

	// Blank iterator keys should error
	_, err := db.ReverseIterator([]byte{}, nil)
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
	db2 := NewBTree()

	itr, err = db2.Iterator(nil, nil)
	require.NoError(t, err)
	verifyIterator(t, itr, nil, "forward iterator with empty db")

	ritr, err = db2.ReverseIterator(nil, nil)
	require.NoError(t, err)
	verifyIterator(t, ritr, nil, "reverse iterator with empty db")
}

func verifyIterator(t *testing.T, itr types.Iterator, expected []int64, msg string) {
	t.Helper()
	i := 0
	for itr.Valid() {
		key := itr.Key()
		require.Equal(t, expected[i], bytes2Int64(key), "iterator: %d mismatches", i)
		itr.Next()
		i++
	}
	require.Equal(t, i, len(expected), "expected to have fully iterated over all the elements in iter")
	require.NoError(t, itr.Close())
}

func int642Bytes(i int64) []byte {
	return types.Uint64ToBigEndian(uint64(i))
}

func bytes2Int64(buf []byte) int64 {
	return int64(types.BigEndianToUint64(buf))
}
