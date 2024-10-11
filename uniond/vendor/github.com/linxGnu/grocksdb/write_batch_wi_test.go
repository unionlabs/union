package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestWriteBatchWI(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	var (
		givenKey1        = []byte("key1")
		givenVal1        = []byte("val1")
		givenKey2        = []byte("key2")
		givenVal2        = []byte("val2")
		givenKey3        = []byte("key3")
		givenVal3        = []byte("val3")
		givenKey4        = []byte("key4")
		givenVal4        = []byte("val4")
		givenVal2Updated = []byte("foo")
	)

	wo := NewDefaultWriteOptions()
	require.Nil(t, db.Put(wo, givenKey1, givenVal1))
	require.Nil(t, db.Put(wo, givenKey2, givenVal2))
	require.Nil(t, db.Put(wo, givenKey3, givenVal3))

	// create and fill the write batch
	wb := NewWriteBatchWI(0, true)
	defer wb.Destroy()
	wb.Put(givenKey2, givenVal2Updated)
	wb.Put(givenKey4, givenVal4)
	wb.Delete(givenKey3)
	require.EqualValues(t, wb.Count(), 3)

	// check before writing to db
	ro := NewDefaultReadOptions()
	v1, err := wb.GetFromDB(db, ro, givenKey1)
	defer v1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), givenVal1)

	v2, err := wb.GetFromDB(db, ro, givenKey2)
	defer v2.Free()
	require.Nil(t, err)
	require.EqualValues(t, v2.Data(), givenVal2Updated)

	v3, err := wb.GetFromDB(db, ro, givenKey3)
	defer v3.Free()
	require.Nil(t, err)
	require.True(t, v3.Data() == nil)

	v4, err := wb.GetFromDB(db, ro, givenKey4)
	defer v4.Free()
	require.Nil(t, err)
	require.EqualValues(t, v4.Data(), givenVal4)

	// perform the batch
	require.Nil(t, db.WriteWI(wo, wb))

	// check changes
	v1, err = db.Get(ro, givenKey1)
	defer v1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), givenVal1)

	v2, err = db.Get(ro, givenKey2)
	defer v2.Free()
	require.Nil(t, err)
	require.EqualValues(t, v2.Data(), givenVal2Updated)

	v3, err = db.Get(ro, givenKey3)
	defer v3.Free()
	require.Nil(t, err)
	require.True(t, v3.Data() == nil)

	v4, err = db.Get(ro, givenKey4)
	defer v4.Free()
	require.Nil(t, err)
	require.EqualValues(t, v4.Data(), givenVal4)

	wb.Clear()
	// DeleteRange not supported
}

func TestWriteBatchWIIterator(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	var (
		givenKey1 = []byte("key1")
		givenVal1 = []byte("val1")
		givenKey2 = []byte("key2")
	)
	// create and fill the write batch
	wb := NewWriteBatchWI(0, true)
	defer wb.Destroy()
	wb.Put(givenKey1, givenVal1)
	wb.Delete(givenKey2)
	require.EqualValues(t, wb.Count(), 2)

	// iterate over the batch
	iter := wb.NewIterator()
	require.True(t, iter.Next())
	record := iter.Record()
	require.EqualValues(t, record.Type, WriteBatchValueRecord)
	require.EqualValues(t, record.Key, givenKey1)
	require.EqualValues(t, record.Value, givenVal1)

	require.True(t, iter.Next())
	record = iter.Record()
	require.EqualValues(t, record.Type, WriteBatchDeletionRecord)
	require.EqualValues(t, record.Key, givenKey2)

	// there shouldn't be any left
	require.False(t, iter.Next())
}

func TestWriteBatchWIIteratorWithBase(t *testing.T) {
	t.Parallel()

	db, cfs, closeup := newTestDBMultiCF(t, []string{"default", "custom"}, nil)
	defer closeup()

	defaultCF := cfs[0]
	customCF := cfs[1]

	var (
		givenKey1        = []byte("key1")
		givenVal1        = []byte("val1")
		givenKey2        = []byte("key2")
		givenVal2        = []byte("val2")
		givenKey3        = []byte("key3")
		givenVal3        = []byte("val3")
		givenKey4        = []byte("key4")
		givenVal4        = []byte("val4")
		givenKey5        = []byte("key5")
		givenVal5        = []byte("val5")
		givenVal2Updated = []byte("foo")
	)

	wo := NewDefaultWriteOptions()
	require.Nil(t, db.PutCF(wo, defaultCF, givenKey1, givenVal1))
	require.Nil(t, db.PutCF(wo, defaultCF, givenKey2, givenVal2))
	require.Nil(t, db.PutCF(wo, defaultCF, givenKey3, givenVal3))
	require.Nil(t, db.PutCF(wo, customCF, givenKey2, givenVal2))
	require.Nil(t, db.PutCF(wo, customCF, givenKey4, givenVal4))

	// create and fill the write batch
	wb := NewWriteBatchWI(0, true)
	defer wb.Destroy()
	wb.PutCF(defaultCF, givenKey2, givenVal2Updated)
	wb.DeleteCF(defaultCF, givenKey3)
	wb.PutCF(defaultCF, givenKey4, givenVal4)
	wb.PutCF(customCF, givenKey5, givenVal5)
	wb.DeleteCF(customCF, givenKey2)
	require.EqualValues(t, wb.Count(), 5)

	// create base iterator for default
	ro := NewDefaultReadOptions()
	defBaseIter := db.NewIteratorCF(ro, defaultCF)

	iter1 := wb.NewIteratorWithBaseCF(db, defBaseIter, defaultCF)
	defer iter1.Close()

	givenKeys1 := [][]byte{givenKey1, givenKey2, givenKey4}
	givenValues1 := [][]byte{givenVal1, givenVal2Updated, givenVal4}

	var actualKeys, actualValues [][]byte

	for iter1.SeekToFirst(); iter1.Valid(); iter1.Next() {
		k := iter1.Key().Data()
		v := iter1.Value().Data()
		key := make([]byte, len(k))
		value := make([]byte, len(v))
		copy(key, k)
		copy(value, v)
		actualKeys = append(actualKeys, key)
		actualValues = append(actualValues, value)
	}

	require.Nil(t, iter1.Err())
	require.EqualValues(t, actualKeys, givenKeys1)
	require.EqualValues(t, actualValues, givenValues1)

	// create base iterator for custom
	customBaseIter := db.NewIteratorCF(ro, customCF)

	iter2 := wb.NewIteratorWithBaseCF(db, customBaseIter, customCF)
	defer iter2.Close()

	givenKeys2 := [][]byte{givenKey4, givenKey5}
	givenValues2 := [][]byte{givenVal4, givenVal5}

	actualKeys = actualKeys[:0]
	actualValues = actualValues[:0]

	for iter2.SeekToFirst(); iter2.Valid(); iter2.Next() {
		k := iter2.Key().Data()
		v := iter2.Value().Data()
		key := make([]byte, len(k))
		value := make([]byte, len(v))
		copy(key, k)
		copy(value, v)
		actualKeys = append(actualKeys, key)
		actualValues = append(actualValues, value)
	}

	require.Nil(t, iter2.Err())
	require.EqualValues(t, actualKeys, givenKeys2)
	require.EqualValues(t, actualValues, givenValues2)
}
