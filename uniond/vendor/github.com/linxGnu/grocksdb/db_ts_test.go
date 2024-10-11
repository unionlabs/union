package grocksdb

import (
	"bytes"
	"encoding/binary"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestDBCRUDWithTS(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, func(opts *Options) {
		opts.SetComparator(newDefaultComparatorWithTS())
	})
	defer db.Close()

	var (
		givenKey  = []byte("hello")
		givenVal1 = []byte("")
		givenVal2 = []byte("world1")

		givenTs1 = marshalTimestamp(1)
		givenTs2 = marshalTimestamp(2)
		givenTs3 = marshalTimestamp(3)
	)

	wo := NewDefaultWriteOptions()
	ro := NewDefaultReadOptions()
	ro.SetTimestamp(givenTs1)

	// create
	require.Nil(t, db.PutWithTS(wo, givenKey, givenTs1, givenVal1))

	// retrieve
	v1, t1, err := db.GetWithTS(ro, givenKey)
	defer v1.Free()
	defer t1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), givenVal1)
	require.EqualValues(t, t1.Data(), givenTs1)

	// retrieve bytes
	_v1, _ts1, err := db.GetBytesWithTS(ro, givenKey)
	require.Nil(t, err)
	require.EqualValues(t, _v1, givenVal1)
	require.EqualValues(t, _ts1, givenTs1)

	// update
	require.Nil(t, db.PutWithTS(wo, givenKey, givenTs2, givenVal2))
	ro.SetTimestamp(givenTs2)
	v2, t2, err := db.GetWithTS(ro, givenKey)
	defer v2.Free()
	defer t2.Free()
	require.Nil(t, err)
	require.EqualValues(t, v2.Data(), givenVal2)
	require.EqualValues(t, t2.Data(), givenTs2)

	// delete
	require.Nil(t, db.DeleteWithTS(wo, givenKey, givenTs3))
	ro.SetTimestamp(givenTs3)
	v3, t3, err := db.GetWithTS(ro, givenKey)
	defer v3.Free()
	defer t3.Free()
	require.Nil(t, err)
	require.True(t, v3.Data() == nil)
	require.True(t, t3.Data() == nil)

	// ts2 should read deleted data
	ro.SetTimestamp(givenTs2)
	v2, t2, err = db.GetWithTS(ro, givenKey)
	defer v2.Free()
	defer t2.Free()
	require.Nil(t, err)
	require.EqualValues(t, v2.Data(), givenVal2)
	require.EqualValues(t, t2.Data(), givenTs2)

	// ts1 should read old data
	ro.SetTimestamp(givenTs1)
	v1, t1, err = db.GetWithTS(ro, givenKey)
	defer v1.Free()
	defer t1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), givenVal1)
	require.EqualValues(t, t1.Data(), givenTs1)
}

func TestDBMultiGetWithTS(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, func(opts *Options) {
		opts.SetComparator(newDefaultComparatorWithTS())
	})
	defer db.Close()

	var (
		givenKey1 = []byte("hello1")
		givenKey2 = []byte("hello2")
		givenKey3 = []byte("hello3")
		givenVal1 = []byte("world1")
		givenVal2 = []byte("world2")
		givenVal3 = []byte("world3")

		givenTs1 = marshalTimestamp(1)
		givenTs2 = marshalTimestamp(2)
		givenTs3 = marshalTimestamp(3)
	)

	wo := NewDefaultWriteOptions()
	ro := NewDefaultReadOptions()
	ro.SetTimestamp(marshalTimestamp(4))

	// create
	require.Nil(t, db.PutWithTS(wo, givenKey1, givenTs1, givenVal1))
	require.Nil(t, db.PutWithTS(wo, givenKey2, givenTs2, givenVal2))
	require.Nil(t, db.PutWithTS(wo, givenKey3, givenTs3, givenVal3))

	// retrieve
	values, times, err := db.MultiGetWithTS(ro, []byte("noexist"), givenKey1, givenKey2, givenKey3)
	defer values.Destroy()
	defer times.Destroy()
	require.Nil(t, err)
	require.EqualValues(t, len(values), 4)

	require.EqualValues(t, values[0].Data(), []byte(nil))
	require.EqualValues(t, values[1].Data(), givenVal1)
	require.EqualValues(t, values[2].Data(), givenVal2)
	require.EqualValues(t, values[3].Data(), givenVal3)

	require.EqualValues(t, times[0].Data(), []byte(nil))
	require.EqualValues(t, times[1].Data(), givenTs1)
	require.EqualValues(t, times[2].Data(), givenTs2)
	require.EqualValues(t, times[3].Data(), givenTs3)
}

const timestampSize = 8

func marshalTimestamp(ts uint64) []byte {
	b := make([]byte, timestampSize)
	binary.BigEndian.PutUint64(b, ts)
	return b
}

func extractUserKey(key []byte) []byte {
	return key[:len(key)-timestampSize]
}

func extractUserTimestamp(key []byte) []byte {
	return key[len(key)-timestampSize:]
}

func extractFromInteralKey(internalKey []byte) (key, timestamp []byte) {
	internalKeySize := 8 // rocksdb internal key size
	userKey := internalKey[:len(internalKey)-internalKeySize]
	key = extractUserKey(userKey)
	timestamp = extractUserTimestamp(userKey)
	return
}

func newDefaultComparatorWithTS() *Comparator {
	return newComparatorWithTimeStamp("default", func(a, b []byte) int {
		return bytes.Compare(a, b)
	})
}

func newComparatorWithTimeStamp(name string, userCompare Comparing) *Comparator {
	compTS := func(a, b []byte) int {
		aTs := binary.BigEndian.Uint64(a)
		bTs := binary.BigEndian.Uint64(b)
		if aTs < bTs {
			return -1
		}
		if aTs > bTs {
			return 1
		}
		return 0
	}

	comp := func(a, b []byte) int {
		aKey := extractUserKey(a)
		bKey := extractUserKey(b)
		res := userCompare(aKey, bKey)
		if res != 0 {
			return res
		}

		aTs := extractUserTimestamp(a)
		bTs := extractUserTimestamp(b)
		return compTS(aTs, bTs) * -1 // timestamp should be reverse ordered
	}

	compWithoutTS := func(a []byte, aHasTs bool, b []byte, bHasTs bool) int {
		var aWithOutTs []byte
		if aHasTs {
			aWithOutTs = extractUserKey(a)
		} else {
			aWithOutTs = a
		}

		var bWithOutTs []byte
		if bHasTs {
			bWithOutTs = extractUserKey(b)
		} else {
			bWithOutTs = b
		}
		return userCompare(aWithOutTs, bWithOutTs)
	}

	return NewComparatorWithTimestamp(name, timestampSize, comp, compTS, compWithoutTS)
}
