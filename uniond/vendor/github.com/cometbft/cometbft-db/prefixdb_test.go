package db

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func mockDBWithStuff(t *testing.T) DB {
	t.Helper()
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
	err = itr.Close()
	require.NoError(t, err)
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
	err = itr.Close()
	require.NoError(t, err)
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
	err = itr.Close()
	require.NoError(t, err)
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
	err = itr.Close()
	require.NoError(t, err)
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
	err = itr.Close()
	require.NoError(t, err)
}
