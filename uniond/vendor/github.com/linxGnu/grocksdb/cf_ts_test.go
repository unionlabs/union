package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestColumnFamilyPutGetDeleteWithTS(t *testing.T) {
	t.Parallel()

	dir := t.TempDir()

	givenNames := []string{"default", "guide"}
	opts := NewDefaultOptions()
	opts.SetCreateIfMissingColumnFamilies(true)
	opts.SetCreateIfMissing(true)
	opts.SetCompression(SnappyCompression)
	opts.SetComparator(newDefaultComparatorWithTS())
	db, cfh, err := OpenDbColumnFamilies(opts, dir, givenNames, []*Options{opts, opts})
	require.Nil(t, err)
	defer db.Close()
	require.EqualValues(t, len(cfh), 2)
	defer cfh[0].Destroy()
	defer cfh[1].Destroy()

	wo := NewDefaultWriteOptions()
	defer wo.Destroy()
	ro := NewDefaultReadOptions()
	defer ro.Destroy()

	givenKey0 := []byte("hello0")
	givenKey1 := []byte("hello1")
	givenVal0 := []byte("world0")
	givenVal1 := []byte("world1")
	givenTs0 := marshalTimestamp(1)
	givenTs1 := marshalTimestamp(2)
	givenTs2 := marshalTimestamp(3)

	{
		ro.SetTimestamp(givenTs2)

		require.Nil(t, db.PutCFWithTS(wo, cfh[0], givenKey0, givenTs0, givenVal0))
		actualVal0, actualTs0, err := db.GetCFWithTS(ro, cfh[0], givenKey0)
		defer actualVal0.Free()
		defer actualTs0.Free()
		require.Nil(t, err)
		require.EqualValues(t, actualVal0.Data(), givenVal0)
		require.EqualValues(t, actualTs0.Data(), givenTs0)

		require.Nil(t, db.PutCFWithTS(wo, cfh[1], givenKey1, givenTs1, givenVal1))
		actualVal1, actualTs1, err := db.GetCFWithTS(ro, cfh[1], givenKey1)
		defer actualVal1.Free()
		defer actualTs1.Free()
		require.Nil(t, err)
		require.EqualValues(t, actualVal1.Data(), givenVal1)
		require.EqualValues(t, actualTs1.Data(), givenTs1)

		actualVal, actualTs, err := db.GetCFWithTS(ro, cfh[0], givenKey1)
		defer actualVal.Free()
		defer actualTs.Free()
		require.Nil(t, err)
		require.EqualValues(t, actualVal.Size(), 0)
		require.EqualValues(t, actualTs.Size(), 0)

		actualVal, actualTs, err = db.GetCFWithTS(ro, cfh[1], givenKey0)
		defer actualVal.Free()
		defer actualTs.Free()
		require.Nil(t, err)
		require.EqualValues(t, actualVal.Size(), 0)
		require.EqualValues(t, actualTs.Size(), 0)

		require.Nil(t, db.DeleteCFWithTS(wo, cfh[0], givenKey0, givenTs2))
		actualVal, actualTs, err = db.GetCFWithTS(ro, cfh[0], givenKey0)
		defer actualVal.Free()
		defer actualTs.Free()
		require.Nil(t, err)
		require.EqualValues(t, actualVal.Size(), 0)
		require.EqualValues(t, actualTs.Size(), 0)
	}

	{
		require.Nil(t, db.PutCFWithTS(wo, cfh[0], givenKey0, givenTs2, givenVal0))
		actualVal0, actualTs0, err := db.GetCFWithTS(ro, cfh[0], givenKey0)
		defer actualVal0.Free()
		defer actualTs0.Free()
		require.Nil(t, err)
		require.EqualValues(t, actualVal0.Data(), givenVal0)
		require.EqualValues(t, actualTs0.Data(), givenTs2)

		actualVal1, actualTs1, err := db.GetCFWithTS(ro, cfh[1], givenKey1)
		defer actualVal1.Free()
		defer actualTs1.Free()
		require.Nil(t, err)
		require.EqualValues(t, actualVal1.Data(), givenVal1)
		require.EqualValues(t, actualTs1.Data(), givenTs1)
	}
}

func TestColumnFamilyMultiGetWithTS(t *testing.T) {
	t.Parallel()

	db, cfh, cleanup := newTestDBMultiCF(t, []string{"default", "custom"}, func(opts *Options) {
		opts.SetComparator(newDefaultComparatorWithTS())
	})
	defer cleanup()

	var (
		givenKey1 = []byte("hello1")
		givenKey2 = []byte("hello2")
		givenKey3 = []byte("hello3")
		givenVal1 = []byte("world1")
		givenVal2 = []byte("world2")
		givenVal3 = []byte("world3")
		givenTs1  = marshalTimestamp(0)
		givenTs2  = marshalTimestamp(2)
		givenTs3  = marshalTimestamp(3)
	)

	wo := NewDefaultWriteOptions()
	defer wo.Destroy()

	ro := NewDefaultReadOptions()
	ro.SetTimestamp(givenTs3)
	defer ro.Destroy()

	// create
	require.Nil(t, db.PutCFWithTS(wo, cfh[0], givenKey1, givenTs1, givenVal1))
	require.Nil(t, db.PutCFWithTS(wo, cfh[1], givenKey2, givenTs2, givenVal2))
	require.Nil(t, db.PutCFWithTS(wo, cfh[1], givenKey3, givenTs3, givenVal3))

	// column family 0 only has givenKey1
	values, times, err := db.MultiGetCFWithTS(ro, cfh[0], []byte("noexist"), givenKey1, givenKey2, givenKey3)
	defer values.Destroy()
	defer times.Destroy()
	require.Nil(t, err)
	require.EqualValues(t, len(values), 4)

	require.EqualValues(t, values[0].Data(), []byte(nil))
	require.EqualValues(t, values[1].Data(), givenVal1)
	require.EqualValues(t, values[2].Data(), []byte(nil))
	require.EqualValues(t, values[3].Data(), []byte(nil))

	require.EqualValues(t, times[0].Data(), []byte(nil))
	require.EqualValues(t, times[1].Data(), givenTs1)
	require.EqualValues(t, times[2].Data(), []byte(nil))
	require.EqualValues(t, times[3].Data(), []byte(nil))

	// column family 1 only has givenKey2 and givenKey3
	values, times, err = db.MultiGetCFWithTS(ro, cfh[1], []byte("noexist"), givenKey1, givenKey2, givenKey3)
	defer values.Destroy()
	defer times.Destroy()
	require.Nil(t, err)
	require.EqualValues(t, len(values), 4)

	require.EqualValues(t, values[0].Data(), []byte(nil))
	require.EqualValues(t, values[1].Data(), []byte(nil))
	require.EqualValues(t, values[2].Data(), givenVal2)
	require.EqualValues(t, values[3].Data(), givenVal3)

	require.EqualValues(t, times[0].Data(), []byte(nil))
	require.EqualValues(t, times[1].Data(), []byte(nil))
	require.EqualValues(t, times[2].Data(), givenTs2)
	require.EqualValues(t, times[3].Data(), givenTs3)

	// getting them all from the right CF should return them all
	values, times, err = db.MultiGetMultiCFWithTS(ro,
		ColumnFamilyHandles{cfh[0], cfh[1], cfh[1]},
		[][]byte{givenKey1, givenKey2, givenKey3},
	)
	defer values.Destroy()
	defer times.Destroy()
	require.Nil(t, err)
	require.EqualValues(t, len(values), 3)

	require.EqualValues(t, values[0].Data(), givenVal1)
	require.EqualValues(t, values[1].Data(), givenVal2)
	require.EqualValues(t, values[2].Data(), givenVal3)

	require.EqualValues(t, times[0].Data(), []byte{})
	require.EqualValues(t, times[1].Data(), givenTs2)
	require.EqualValues(t, times[2].Data(), givenTs3)
}
