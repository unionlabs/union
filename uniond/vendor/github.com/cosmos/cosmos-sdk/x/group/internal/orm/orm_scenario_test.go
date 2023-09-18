package orm

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"testing"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

// Testing ORM with arbitrary metadata length
const metadataLen = 10

func TestKeeperEndToEndWithAutoUInt64Table(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	ctx := NewMockContext()
	store := ctx.KVStore(sdk.NewKVStoreKey("test"))

	k := NewTestKeeper(cdc)

	tm := testdata.TableModel{
		Id:       1,
		Name:     "name",
		Number:   123,
		Metadata: []byte("metadata"),
	}
	// when stored
	rowID, err := k.autoUInt64Table.Create(store, &tm)
	require.NoError(t, err)
	// then we should find it
	exists := k.autoUInt64Table.Has(store, rowID)
	require.True(t, exists)

	// and load it
	var loaded testdata.TableModel

	binKey, err := k.autoUInt64Table.GetOne(store, rowID, &loaded)
	require.NoError(t, err)

	require.Equal(t, rowID, binary.BigEndian.Uint64(binKey))
	require.Equal(t, tm, loaded)

	// and exists in MultiKeyIndex
	exists, err = k.autoUInt64TableModelByMetadataIndex.Has(store, []byte("metadata"))
	require.NoError(t, err)
	require.True(t, exists)

	// and when loaded
	it, err := k.autoUInt64TableModelByMetadataIndex.Get(store, []byte("metadata"))
	require.NoError(t, err)

	// then
	binKey, loaded = first(t, it)
	assert.Equal(t, rowID, binary.BigEndian.Uint64(binKey))
	assert.Equal(t, tm, loaded)

	// when updated
	tm.Metadata = []byte("new-metadata")
	err = k.autoUInt64Table.Update(store, rowID, &tm)
	require.NoError(t, err)

	binKey, err = k.autoUInt64Table.GetOne(store, rowID, &loaded)
	require.NoError(t, err)

	require.Equal(t, rowID, binary.BigEndian.Uint64(binKey))
	require.Equal(t, tm, loaded)

	// then indexes are updated, too
	exists, err = k.autoUInt64TableModelByMetadataIndex.Has(store, []byte("new-metadata"))
	require.NoError(t, err)
	require.True(t, exists)

	exists, err = k.autoUInt64TableModelByMetadataIndex.Has(store, []byte("metadata"))
	require.NoError(t, err)
	require.False(t, exists)

	// when deleted
	err = k.autoUInt64Table.Delete(store, rowID)
	require.NoError(t, err)

	exists = k.autoUInt64Table.Has(store, rowID)
	require.False(t, exists)

	// and also removed from secondary MultiKeyIndex
	exists, err = k.autoUInt64TableModelByMetadataIndex.Has(store, []byte("new-metadata"))
	require.NoError(t, err)
	require.False(t, exists)
}

func TestKeeperEndToEndWithPrimaryKeyTable(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	ctx := NewMockContext()
	store := ctx.KVStore(sdk.NewKVStoreKey("test"))

	k := NewTestKeeper(cdc)

	tm := testdata.TableModel{
		Id:       1,
		Name:     "name",
		Number:   123,
		Metadata: []byte("metadata"),
	}
	// when stored
	err := k.primaryKeyTable.Create(store, &tm)
	require.NoError(t, err)
	// then we should find it by primary key
	primaryKey := PrimaryKey(&tm)
	exists := k.primaryKeyTable.Has(store, primaryKey)
	require.True(t, exists)

	// and load it by primary key
	var loaded testdata.TableModel
	err = k.primaryKeyTable.GetOne(store, primaryKey, &loaded)
	require.NoError(t, err)

	// then values should match expectations
	require.Equal(t, tm, loaded)

	// and then the data should exists in MultiKeyIndex
	exists, err = k.primaryKeyTableModelByNumberIndex.Has(store, tm.Number)
	require.NoError(t, err)
	require.True(t, exists)

	// and when loaded from MultiKeyIndex
	it, err := k.primaryKeyTableModelByNumberIndex.Get(store, tm.Number)
	require.NoError(t, err)

	// then values should match as before
	_, err = First(it, &loaded)
	require.NoError(t, err)
	assert.Equal(t, tm, loaded)

	// and when we create another entry with the same primary key
	err = k.primaryKeyTable.Create(store, &tm)
	// then it should fail as the primary key must be unique
	require.True(t, errors.ErrORMUniqueConstraint.Is(err), err)

	// and when entity updated with new primary key
	updatedMember := &testdata.TableModel{
		Id:       2,
		Name:     tm.Name,
		Number:   tm.Number,
		Metadata: tm.Metadata,
	}
	// then it should fail as the primary key is immutable
	err = k.primaryKeyTable.Update(store, updatedMember)
	require.Error(t, err)

	// and when entity updated with non primary key attribute modified
	updatedMember = &testdata.TableModel{
		Id:       1,
		Name:     "new name",
		Number:   tm.Number,
		Metadata: tm.Metadata,
	}
	// then it should not fail
	err = k.primaryKeyTable.Update(store, updatedMember)
	require.NoError(t, err)

	// and when entity deleted
	err = k.primaryKeyTable.Delete(store, &tm)
	require.NoError(t, err)

	// it is removed from primaryKeyTable
	exists = k.primaryKeyTable.Has(store, primaryKey)
	require.False(t, exists)

	// and removed from secondary MultiKeyIndex
	exists, err = k.primaryKeyTableModelByNumberIndex.Has(store, tm.Number)
	require.NoError(t, err)
	require.False(t, exists)
}

func TestGasCostsPrimaryKeyTable(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	ctx := NewMockContext()
	store := ctx.KVStore(sdk.NewKVStoreKey("test"))

	k := NewTestKeeper(cdc)

	tm := testdata.TableModel{
		Id:       1,
		Name:     "name",
		Number:   123,
		Metadata: []byte("metadata"),
	}
	rowID, err := k.autoUInt64Table.Create(store, &tm)
	require.NoError(t, err)
	require.Equal(t, uint64(1), rowID)

	gCtx := NewGasCountingMockContext()
	err = k.primaryKeyTable.Create(gCtx.KVStore(store), &tm)
	require.NoError(t, err)
	t.Logf("gas consumed on create: %d", gCtx.GasConsumed())

	// get by primary key
	gCtx.ResetGasMeter()
	var loaded testdata.TableModel
	err = k.primaryKeyTable.GetOne(gCtx.KVStore(store), PrimaryKey(&tm), &loaded)
	require.NoError(t, err)
	t.Logf("gas consumed on get by primary key: %d", gCtx.GasConsumed())

	// get by secondary index
	gCtx.ResetGasMeter()
	// and when loaded from MultiKeyIndex
	it, err := k.primaryKeyTableModelByNumberIndex.Get(gCtx.KVStore(store), tm.Number)
	require.NoError(t, err)
	var loadedSlice []testdata.TableModel
	_, err = ReadAll(it, &loadedSlice)
	require.NoError(t, err)
	t.Logf("gas consumed on get by multi index key: %d", gCtx.GasConsumed())

	// delete
	gCtx.ResetGasMeter()
	err = k.primaryKeyTable.Delete(gCtx.KVStore(store), &tm)
	require.NoError(t, err)
	t.Logf("gas consumed on delete by primary key: %d", gCtx.GasConsumed())

	// with 3 elements
	var tms []testdata.TableModel
	for i := 1; i < 4; i++ {
		gCtx.ResetGasMeter()
		tm := testdata.TableModel{
			Id:       uint64(i),
			Name:     fmt.Sprintf("name%d", i),
			Number:   123,
			Metadata: []byte("metadata"),
		}
		err = k.primaryKeyTable.Create(gCtx.KVStore(store), &tm)
		require.NoError(t, err)
		t.Logf("%d: gas consumed on create: %d", i, gCtx.GasConsumed())
		tms = append(tms, tm)
	}

	for i := 1; i < 4; i++ {
		gCtx.ResetGasMeter()
		tm := testdata.TableModel{
			Id:       uint64(i),
			Name:     fmt.Sprintf("name%d", i),
			Number:   123,
			Metadata: []byte("metadata"),
		}
		err = k.primaryKeyTable.GetOne(gCtx.KVStore(store), PrimaryKey(&tm), &loaded)
		require.NoError(t, err)
		t.Logf("%d: gas consumed on get by primary key: %d", i, gCtx.GasConsumed())
	}

	// get by secondary index
	gCtx.ResetGasMeter()
	// and when loaded from MultiKeyIndex
	it, err = k.primaryKeyTableModelByNumberIndex.Get(gCtx.KVStore(store), tm.Number)
	require.NoError(t, err)
	_, err = ReadAll(it, &loadedSlice)
	require.NoError(t, err)
	require.Len(t, loadedSlice, 3)
	t.Logf("gas consumed on get by multi index key: %d", gCtx.GasConsumed())

	// delete
	for i, m := range tms {
		gCtx.ResetGasMeter()

		err = k.primaryKeyTable.Delete(gCtx.KVStore(store), &m)
		require.NoError(t, err)
		t.Logf("%d: gas consumed on delete: %d", i, gCtx.GasConsumed())
	}
}

func TestExportImportStateAutoUInt64Table(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	ctx := NewMockContext()
	store := ctx.KVStore(sdk.NewKVStoreKey("test"))

	k := NewTestKeeper(cdc)

	testRecordsNum := 10
	for i := 1; i <= testRecordsNum; i++ {
		tm := testdata.TableModel{
			Id:       uint64(i),
			Name:     fmt.Sprintf("my test %d", i),
			Metadata: bytes.Repeat([]byte{byte(i)}, metadataLen),
		}

		rowID, err := k.autoUInt64Table.Create(store, &tm)
		require.NoError(t, err)
		require.Equal(t, uint64(i), rowID)
	}
	var tms []*testdata.TableModel
	seqVal, err := k.autoUInt64Table.Export(store, &tms)
	require.NoError(t, err)
	require.Equal(t, seqVal, uint64(testRecordsNum))

	// when a new db seeded
	ctx = NewMockContext()
	store = ctx.KVStore(sdk.NewKVStoreKey("test"))

	err = k.autoUInt64Table.Import(store, tms, seqVal)
	require.NoError(t, err)

	// then all data is set again
	for i := 1; i <= testRecordsNum; i++ {
		require.True(t, k.autoUInt64Table.Has(store, uint64(i)))
		var loaded testdata.TableModel
		rowID, err := k.autoUInt64Table.GetOne(store, uint64(i), &loaded)
		require.NoError(t, err)

		require.Equal(t, RowID(EncodeSequence(uint64(i))), rowID)
		assert.Equal(t, fmt.Sprintf("my test %d", i), loaded.Name)
		exp := bytes.Repeat([]byte{byte(i)}, metadataLen)
		assert.Equal(t, exp, loaded.Metadata)

		// and also the indexes
		exists, err := k.autoUInt64TableModelByMetadataIndex.Has(store, exp)
		require.NoError(t, err)
		require.True(t, exists)

		it, err := k.autoUInt64TableModelByMetadataIndex.Get(store, exp)
		require.NoError(t, err)
		var all []testdata.TableModel
		ReadAll(it, &all)
		require.Len(t, all, 1)
		assert.Equal(t, loaded, all[0])
	}
	require.Equal(t, uint64(testRecordsNum), k.autoUInt64Table.Sequence().CurVal(store))
}

func TestExportImportStatePrimaryKeyTable(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	ctx := NewMockContext()
	store := ctx.KVStore(sdk.NewKVStoreKey("test"))

	k := NewTestKeeper(cdc)

	testRecordsNum := 10
	testRecords := make([]testdata.TableModel, testRecordsNum)
	for i := 1; i <= testRecordsNum; i++ {
		tm := testdata.TableModel{
			Id:       uint64(i),
			Name:     fmt.Sprintf("my test %d", i),
			Number:   uint64(i - 1),
			Metadata: bytes.Repeat([]byte{byte(i)}, metadataLen),
		}

		err := k.primaryKeyTable.Create(store, &tm)
		require.NoError(t, err)
		testRecords[i-1] = tm
	}
	var tms []*testdata.TableModel
	_, err := k.primaryKeyTable.Export(store, &tms)
	require.NoError(t, err)

	// when a new db seeded
	ctx = NewMockContext()
	store = ctx.KVStore(sdk.NewKVStoreKey("test"))

	err = k.primaryKeyTable.Import(store, tms, 0)
	require.NoError(t, err)

	// then all data is set again
	it, err := k.primaryKeyTable.PrefixScan(store, nil, nil)
	require.NoError(t, err)
	var loaded []testdata.TableModel
	keys, err := ReadAll(it, &loaded)
	require.NoError(t, err)
	for i := range keys {
		assert.Equal(t, PrimaryKey(&testRecords[i]), keys[i].Bytes())
	}
	assert.Equal(t, testRecords, loaded)

	// all indexes setup
	for _, v := range testRecords {
		assertIndex(t, store, k.primaryKeyTableModelByNameIndex, v, v.Name)
		assertIndex(t, store, k.primaryKeyTableModelByNumberIndex, v, v.Number)
		assertIndex(t, store, k.primaryKeyTableModelByMetadataIndex, v, v.Metadata)
	}
}

func assertIndex(t *testing.T, store sdk.KVStore, index Index, v testdata.TableModel, searchKey interface{}) {
	it, err := index.Get(store, searchKey)
	require.NoError(t, err)

	var loaded []testdata.TableModel
	keys, err := ReadAll(it, &loaded)
	require.NoError(t, err)
	assert.Equal(t, []RowID{PrimaryKey(&v)}, keys)
	assert.Equal(t, []testdata.TableModel{v}, loaded)
}

func first(t *testing.T, it Iterator) ([]byte, testdata.TableModel) {
	var loaded testdata.TableModel
	key, err := First(it, &loaded)
	require.NoError(t, err)
	return key, loaded
}
