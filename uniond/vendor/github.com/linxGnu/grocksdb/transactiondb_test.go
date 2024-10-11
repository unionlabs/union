package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestOpenTransactionDb(t *testing.T) {
	t.Parallel()

	db := newTestTransactionDB(t, nil)
	defer db.Close()
}

func TestTransactionDBCRUD(t *testing.T) {
	t.Parallel()

	db := newTestTransactionDB(t, nil)
	defer db.Close()

	var (
		givenKey     = []byte("hello")
		givenVal1    = []byte("world1")
		givenVal2    = []byte("world2")
		givenTxnKey  = []byte("hello2")
		givenTxnKey2 = []byte("hello3")
		givenTxnVal1 = []byte("whatawonderful")
		wo           = NewDefaultWriteOptions()
		ro           = NewDefaultReadOptions()
		to           = NewDefaultTransactionOptions()
	)

	// create
	require.Nil(t, db.Put(wo, givenKey, givenVal1))

	// retrieve
	v1, err := db.Get(ro, givenKey)
	defer v1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), givenVal1)

	// update
	require.Nil(t, db.Put(wo, givenKey, givenVal2))
	v2, err := db.Get(ro, givenKey)
	defer v2.Free()
	require.Nil(t, err)
	require.EqualValues(t, v2.Data(), givenVal2)

	// delete
	require.Nil(t, db.Delete(wo, givenKey))
	v3, err := db.Get(ro, givenKey)
	defer v3.Free()
	require.Nil(t, err)
	require.True(t, v3.Data() == nil)

	// transaction
	txn := db.TransactionBegin(wo, to, nil)
	defer txn.Destroy()
	// create
	require.Nil(t, txn.Put(givenTxnKey, givenTxnVal1))
	v4, err := txn.Get(ro, givenTxnKey)
	defer v4.Free()
	require.Nil(t, err)
	require.EqualValues(t, v4.Data(), givenTxnVal1)

	require.Nil(t, txn.Commit())
	v5, err := db.Get(ro, givenTxnKey)
	defer v5.Free()
	require.Nil(t, err)
	require.EqualValues(t, v5.Data(), givenTxnVal1)

	// transaction
	txn2 := db.TransactionBegin(wo, to, nil)
	defer txn2.Destroy()
	// create
	require.Nil(t, txn2.Put(givenTxnKey2, givenTxnVal1))
	// rollback
	require.Nil(t, txn2.Rollback())
	v6, err := txn2.Get(ro, givenTxnKey2)
	defer v6.Free()
	require.Nil(t, err)
	require.True(t, v6.Data() == nil)

	// transaction
	txn3 := db.TransactionBegin(wo, to, nil)
	defer txn3.Destroy()
	require.NoError(t, txn3.SetName("fake_name"))
	require.Equal(t, "fake_name", txn3.GetName())
	// delete
	require.Nil(t, txn3.Prepare())
	require.Nil(t, txn3.Delete(givenTxnKey))

	wi := txn3.GetWriteBatchWI()
	require.EqualValues(t, 1, wi.Count())

	require.Nil(t, txn3.Commit())

	v7, err := db.Get(ro, givenTxnKey)
	defer v7.Free()
	require.Nil(t, err)
	require.True(t, v7.Data() == nil)

	// transaction
	txn4 := db.TransactionBegin(wo, to, nil)
	defer txn4.Destroy()

	// mark delete
	require.Nil(t, txn4.Delete(givenTxnKey))

	// rebuild with put op
	wi = NewWriteBatchWI(123, true)
	wi.Put(givenTxnKey, givenTxnVal1)
	require.Nil(t, txn4.RebuildFromWriteBatchWI(wi))
	require.Nil(t, txn4.Commit())

	v8, err := db.Get(ro, givenTxnKey)
	defer v8.Free()
	require.Nil(t, err)
	require.True(t, v8.Data() != nil) // due to rebuild -> put -> key exists

	// transaction
	txn5 := db.TransactionBegin(wo, to, nil)
	defer txn5.Destroy()

	// mark delete
	require.Nil(t, txn5.Delete(givenTxnKey2))

	// rebuild with put op
	wb := NewWriteBatch()
	wb.Put(givenTxnKey2, givenTxnVal1)
	require.Nil(t, txn5.RebuildFromWriteBatch(wb))

	v, err := txn5.GetPinned(ro, givenTxnKey2)
	require.Nil(t, err)
	require.Equal(t, []byte(givenTxnVal1), v.Data())
	v.Destroy()

	require.Nil(t, txn5.Commit())

	v9, err := db.Get(ro, givenTxnKey2)
	defer v9.Free()
	require.Nil(t, err)
	require.True(t, v8.Data() != nil) // due to rebuild -> put -> key exists
}

func TestTransactionDBGetForUpdate(t *testing.T) {
	t.Parallel()

	lockTimeoutMilliSec := int64(50)
	applyOpts := func(_ *Options, transactionDBOpts *TransactionDBOptions) {
		transactionDBOpts.SetTransactionLockTimeout(lockTimeoutMilliSec)
	}
	db := newTestTransactionDB(t, applyOpts)
	defer db.Close()

	var (
		givenKey = []byte("hello")
		givenVal = []byte("world")
		wo       = NewDefaultWriteOptions()
		ro       = NewDefaultReadOptions()
		to       = NewDefaultTransactionOptions()
	)

	txn := db.TransactionBegin(wo, to, nil)
	defer txn.Destroy()

	v, err := txn.GetForUpdate(ro, givenKey)
	defer v.Free()
	require.Nil(t, err)

	// expect lock timeout error to be thrown
	if err := db.Put(wo, givenKey, givenVal); err == nil {
		t.Error("expect locktime out error, got nil error")
	}

	base := db.GetBaseDB()
	defer CloseBaseDBOfTransactionDB(base)
}

func newTestTransactionDB(t *testing.T, applyOpts func(opts *Options, transactionDBOpts *TransactionDBOptions)) *TransactionDB {
	dir := t.TempDir()

	opts := NewDefaultOptions()
	opts.SetCreateIfMissing(true)
	transactionDBOpts := NewDefaultTransactionDBOptions()
	if applyOpts != nil {
		applyOpts(opts, transactionDBOpts)
	}
	db, err := OpenTransactionDb(opts, transactionDBOpts, dir)
	require.Nil(t, err)

	return db
}

func TestTransactionDBColumnFamilyBatchPutGet(t *testing.T) {
	t.Parallel()

	dir := t.TempDir()

	givenNames := []string{"default", "guide"}

	opts := NewDefaultOptions()
	opts.SetCreateIfMissingColumnFamilies(true)
	opts.SetCreateIfMissing(true)

	db, cfh, err := OpenTransactionDbColumnFamilies(opts, NewDefaultTransactionDBOptions(), dir, givenNames, []*Options{opts, opts})
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
	givenVal0 := []byte("world0")
	givenKey1 := []byte("hello1")
	givenVal1 := []byte("world1")
	givenKey2 := []byte("hello2")
	givenVal2 := []byte("world2")

	writeReadBatch := func(cf *ColumnFamilyHandle, keys [][]byte, values [][]byte) {
		b := NewWriteBatch()
		defer b.Destroy()
		for i := range keys {
			b.PutCF(cf, keys[i], values[i])
		}
		require.Nil(t, db.Write(wo, b))

		for i := range keys {
			actualVal, err := db.GetCF(ro, cf, keys[i])
			require.Nil(t, err)
			require.EqualValues(t, actualVal.Data(), values[i])
			actualVal.Free()
		}
	}

	writeReadBatch(cfh[0], [][]byte{givenKey0}, [][]byte{givenVal0})

	writeReadBatch(cfh[1], [][]byte{givenKey1, givenKey2}, [][]byte{givenVal1, givenVal2})

	// check read from wrong CF returns nil
	actualVal, err := db.GetCF(ro, cfh[0], givenKey1)
	require.Nil(t, err)
	require.EqualValues(t, actualVal.Size(), 0)
	actualVal.Free()

	actualVal, err = db.GetCF(ro, cfh[1], givenKey0)
	require.Nil(t, err)
	require.EqualValues(t, actualVal.Size(), 0)
	actualVal.Free()

	// check batch read is correct
	actualVals, err := db.MultiGetWithCF(ro, cfh[1], givenKey1, givenKey2)
	require.Nil(t, err)
	require.EqualValues(t, len(actualVals), 2)
	require.EqualValues(t, actualVals[0].Data(), givenVal1)
	require.EqualValues(t, actualVals[1].Data(), givenVal2)
	actualVals.Destroy()

	// trigger flush
	require.Nil(t, db.FlushCF(cfh[0], NewDefaultFlushOptions()))
	require.Nil(t, db.FlushCFs(cfh, NewDefaultFlushOptions()))
}
