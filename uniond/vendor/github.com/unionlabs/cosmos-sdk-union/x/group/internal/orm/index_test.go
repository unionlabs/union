package orm

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	corestore "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/group/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/types/query"
)

var _ Indexable = &nilRowGetterBuilder{}

type nilRowGetterBuilder struct{}

func (b *nilRowGetterBuilder) RowGetter() RowGetter {
	return nil
}
func (b *nilRowGetterBuilder) AddAfterSetInterceptor(AfterSetInterceptor)       {}
func (b *nilRowGetterBuilder) AddAfterDeleteInterceptor(AfterDeleteInterceptor) {}

func TestNewIndex(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	myTable, err := NewAutoUInt64Table(AutoUInt64TablePrefix, AutoUInt64TableSeqPrefix, &testdata.TableModel{}, cdc, address.NewBech32Codec("cosmos"))
	require.NoError(t, err)
	indexer := func(val interface{}) ([]interface{}, error) {
		return []interface{}{val.(*testdata.TableModel).Metadata}, nil
	}

	testCases := []struct {
		name        string
		table       Indexable
		expectErr   bool
		expectedErr string
		indexKey    interface{}
	}{
		{
			name:        "nil indexKey",
			table:       myTable,
			expectErr:   true,
			expectedErr: "indexKey must not be nil",
			indexKey:    nil,
		},
		{
			name:        "nil rowGetter",
			table:       &nilRowGetterBuilder{},
			expectErr:   true,
			expectedErr: "rowGetter must not be nil",
			indexKey:    []byte{},
		},
		{
			name:      "all not nil",
			table:     myTable,
			expectErr: false,
			indexKey:  []byte{},
		},
		{
			name:      "index key type not allowed",
			table:     myTable,
			expectErr: true,
			indexKey:  1,
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			index, err := NewIndex(tc.table, AutoUInt64TableSeqPrefix, indexer, tc.indexKey)
			if tc.expectErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expectedErr)
			} else {
				require.NoError(t, err)
				require.NotEmpty(t, index)
			}
		})
	}
}

func TestIndexPrefixScan(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	tb, err := NewAutoUInt64Table(AutoUInt64TablePrefix, AutoUInt64TableSeqPrefix, &testdata.TableModel{}, cdc, address.NewBech32Codec("cosmos"))
	require.NoError(t, err)
	idx, err := NewIndex(tb, AutoUInt64TableModelByMetadataPrefix, func(val interface{}) ([]interface{}, error) {
		i := []interface{}{val.(*testdata.TableModel).Metadata}
		return i, nil
	}, testdata.TableModel{}.Metadata)
	require.NoError(t, err)
	strIdx, err := NewIndex(tb, 0x1, func(val interface{}) ([]interface{}, error) {
		i := []interface{}{val.(*testdata.TableModel).Name}
		return i, nil
	}, testdata.TableModel{}.Name)
	require.NoError(t, err)

	key := storetypes.NewKVStoreKey("test")
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)

	g1 := testdata.TableModel{
		Id:       1,
		Name:     "my test 1",
		Metadata: []byte("metadata-a"),
	}
	g2 := testdata.TableModel{
		Id:       2,
		Name:     "my test 2",
		Metadata: []byte("metadata-b"),
	}
	g3 := testdata.TableModel{
		Id:       3,
		Name:     "my test 3",
		Metadata: []byte("metadata-b"),
	}
	for _, g := range []testdata.TableModel{g1, g2, g3} {
		_, err := tb.Create(store, &g)
		require.NoError(t, err)
	}

	specs := map[string]struct {
		start, end interface{}
		expResult  []testdata.TableModel
		expRowIDs  []RowID
		expError   *errorsmod.Error
		method     func(store corestore.KVStore, start, end interface{}) (Iterator, error)
	}{
		"exact match with a single result": {
			start:     []byte("metadata-a"),
			end:       []byte("metadata-b"),
			method:    idx.PrefixScan,
			expResult: []testdata.TableModel{g1},
			expRowIDs: []RowID{EncodeSequence(1)},
		},
		"one result by prefix": {
			start:     []byte("metadata"),
			end:       []byte("metadata-b"),
			method:    idx.PrefixScan,
			expResult: []testdata.TableModel{g1},
			expRowIDs: []RowID{EncodeSequence(1)},
		},
		"multi key elements by exact match": {
			start:     []byte("metadata-b"),
			end:       []byte("metadata-c"),
			method:    idx.PrefixScan,
			expResult: []testdata.TableModel{g2, g3},
			expRowIDs: []RowID{EncodeSequence(2), EncodeSequence(3)},
		},
		"open end query": {
			start:     []byte("metadata-b"),
			end:       nil,
			method:    idx.PrefixScan,
			expResult: []testdata.TableModel{g2, g3},
			expRowIDs: []RowID{EncodeSequence(2), EncodeSequence(3)},
		},
		"open start query": {
			start:     nil,
			end:       []byte("metadata-b"),
			method:    idx.PrefixScan,
			expResult: []testdata.TableModel{g1},
			expRowIDs: []RowID{EncodeSequence(1)},
		},
		"open start and end query": {
			start:     nil,
			end:       nil,
			method:    idx.PrefixScan,
			expResult: []testdata.TableModel{g1, g2, g3},
			expRowIDs: []RowID{EncodeSequence(1), EncodeSequence(2), EncodeSequence(3)},
		},
		"all matching prefix": {
			start:     []byte("admin"),
			end:       nil,
			method:    idx.PrefixScan,
			expResult: []testdata.TableModel{g1, g2, g3},
			expRowIDs: []RowID{EncodeSequence(1), EncodeSequence(2), EncodeSequence(3)},
		},
		"non matching prefix": {
			start:     []byte("metadata-c"),
			end:       nil,
			method:    idx.PrefixScan,
			expResult: []testdata.TableModel{},
		},
		"start equals end": {
			start:    []byte("any"),
			end:      []byte("any"),
			method:   idx.PrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"start after end": {
			start:    []byte("b"),
			end:      []byte("a"),
			method:   idx.PrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"reverse: exact match with a single result": {
			start:     []byte("metadata-a"),
			end:       []byte("metadata-b"),
			method:    idx.ReversePrefixScan,
			expResult: []testdata.TableModel{g1},
			expRowIDs: []RowID{EncodeSequence(1)},
		},
		"reverse: one result by prefix": {
			start:     []byte("metadata"),
			end:       []byte("metadata-b"),
			method:    idx.ReversePrefixScan,
			expResult: []testdata.TableModel{g1},
			expRowIDs: []RowID{EncodeSequence(1)},
		},
		"reverse: multi key elements by exact match": {
			start:     []byte("metadata-b"),
			end:       []byte("metadata-c"),
			method:    idx.ReversePrefixScan,
			expResult: []testdata.TableModel{g3, g2},
			expRowIDs: []RowID{EncodeSequence(3), EncodeSequence(2)},
		},
		"reverse: open end query": {
			start:     []byte("metadata-b"),
			end:       nil,
			method:    idx.ReversePrefixScan,
			expResult: []testdata.TableModel{g3, g2},
			expRowIDs: []RowID{EncodeSequence(3), EncodeSequence(2)},
		},
		"reverse: open start query": {
			start:     nil,
			end:       []byte("metadata-b"),
			method:    idx.ReversePrefixScan,
			expResult: []testdata.TableModel{g1},
			expRowIDs: []RowID{EncodeSequence(1)},
		},
		"reverse: open start and end query": {
			start:     nil,
			end:       nil,
			method:    idx.ReversePrefixScan,
			expResult: []testdata.TableModel{g3, g2, g1},
			expRowIDs: []RowID{EncodeSequence(3), EncodeSequence(2), EncodeSequence(1)},
		},
		"reverse: all matching prefix": {
			start:     []byte("admin"),
			end:       nil,
			method:    idx.ReversePrefixScan,
			expResult: []testdata.TableModel{g3, g2, g1},
			expRowIDs: []RowID{EncodeSequence(3), EncodeSequence(2), EncodeSequence(1)},
		},
		"reverse: non matching prefix": {
			start:     []byte("metadata-c"),
			end:       nil,
			method:    idx.ReversePrefixScan,
			expResult: []testdata.TableModel{},
		},
		"reverse: start equals end": {
			start:    []byte("any"),
			end:      []byte("any"),
			method:   idx.ReversePrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"reverse: start after end": {
			start:    []byte("b"),
			end:      []byte("a"),
			method:   idx.ReversePrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"exact match with a single result using string based index": {
			start:     "my test 1",
			end:       "my test 2",
			method:    strIdx.PrefixScan,
			expResult: []testdata.TableModel{g1},
			expRowIDs: []RowID{EncodeSequence(1)},
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			it, err := spec.method(store, spec.start, spec.end)
			require.True(t, spec.expError.Is(err), "expected #+v but got #+v", spec.expError, err)
			if spec.expError != nil {
				return
			}
			var loaded []testdata.TableModel
			rowIDs, err := ReadAll(it, &loaded)
			require.NoError(t, err)
			assert.Equal(t, spec.expResult, loaded)
			assert.Equal(t, spec.expRowIDs, rowIDs)
		})
	}
}

func TestUniqueIndex(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)
	ac := address.NewBech32Codec("cosmos")
	myTable, err := NewPrimaryKeyTable(PrimaryKeyTablePrefix, &testdata.TableModel{}, cdc, ac)
	require.NoError(t, err)
	uniqueIdx, err := NewUniqueIndex(myTable, 0x10, func(val interface{}) (interface{}, error) {
		return []byte{val.(*testdata.TableModel).Metadata[0]}, nil
	}, []byte{})
	require.NoError(t, err)

	key := storetypes.NewKVStoreKey("test")
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)

	m := testdata.TableModel{
		Id:       1,
		Name:     "my test",
		Metadata: []byte("metadata"),
	}
	err = myTable.Create(store, &m)
	require.NoError(t, err)

	indexedKey := []byte{'m'}

	// Has
	exists, err := uniqueIdx.Has(store, indexedKey)
	require.NoError(t, err)
	assert.True(t, exists)

	// Get
	it, err := uniqueIdx.Get(store, indexedKey)
	require.NoError(t, err)
	var loaded testdata.TableModel
	rowID, err := it.LoadNext(&loaded)
	require.NoError(t, err)
	require.Equal(t, RowID(PrimaryKey(&m, ac)), rowID)
	require.Equal(t, m, loaded)

	// GetPaginated
	cases := map[string]struct {
		pageReq *query.PageRequest
		expErr  bool
	}{
		"nil key": {
			pageReq: &query.PageRequest{Key: nil},
			expErr:  false,
		},
		"after indexed key": {
			pageReq: &query.PageRequest{Key: indexedKey},
			expErr:  true,
		},
	}

	for testName, tc := range cases {
		t.Run(testName, func(t *testing.T) {
			it, err := uniqueIdx.GetPaginated(store, indexedKey, tc.pageReq)
			require.NoError(t, err)
			rowID, err := it.LoadNext(&loaded)
			if tc.expErr { // iterator done
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				require.Equal(t, RowID(PrimaryKey(&m, ac)), rowID)
				require.Equal(t, m, loaded)
			}
		})
	}

	// PrefixScan match
	it, err = uniqueIdx.PrefixScan(store, indexedKey, nil)
	require.NoError(t, err)
	rowID, err = it.LoadNext(&loaded)
	require.NoError(t, err)
	require.Equal(t, RowID(PrimaryKey(&m, ac)), rowID)
	require.Equal(t, m, loaded)

	// PrefixScan no match
	it, err = uniqueIdx.PrefixScan(store, []byte{byte('n')}, nil)
	require.NoError(t, err)
	_, err = it.LoadNext(&loaded)
	require.Error(t, errors.ErrORMIteratorDone, err)

	// ReversePrefixScan match
	it, err = uniqueIdx.ReversePrefixScan(store, indexedKey, nil)
	require.NoError(t, err)
	rowID, err = it.LoadNext(&loaded)
	require.NoError(t, err)
	require.Equal(t, RowID(PrimaryKey(&m, ac)), rowID)
	require.Equal(t, m, loaded)

	// ReversePrefixScan no match
	it, err = uniqueIdx.ReversePrefixScan(store, []byte{byte('l')}, nil)
	require.NoError(t, err)
	_, err = it.LoadNext(&loaded)
	require.Error(t, errors.ErrORMIteratorDone, err)
	// create with same index key should fail
	new := testdata.TableModel{
		Id:       1,
		Name:     "my test",
		Metadata: []byte("my-metadata"),
	}
	err = myTable.Create(store, &new)
	require.Error(t, errors.ErrORMUniqueConstraint, err)

	// and when delete
	err = myTable.Delete(store, &m)
	require.NoError(t, err)

	// then no persistent element
	exists, err = uniqueIdx.Has(store, indexedKey)
	require.NoError(t, err)
	assert.False(t, exists)
}

func TestPrefixRange(t *testing.T) {
	cases := map[string]struct {
		src      []byte
		expStart []byte
		expEnd   []byte
		expPanic bool
	}{
		"normal":                 {src: []byte{1, 3, 4}, expStart: []byte{1, 3, 4}, expEnd: []byte{1, 3, 5}},
		"normal short":           {src: []byte{79}, expStart: []byte{79}, expEnd: []byte{80}},
		"empty case":             {src: []byte{}},
		"roll-over example 1":    {src: []byte{17, 28, 255}, expStart: []byte{17, 28, 255}, expEnd: []byte{17, 29, 0}},
		"roll-over example 2":    {src: []byte{15, 42, 255, 255}, expStart: []byte{15, 42, 255, 255}, expEnd: []byte{15, 43, 0, 0}},
		"pathological roll-over": {src: []byte{255, 255, 255, 255}, expStart: []byte{255, 255, 255, 255}},
		"nil prohibited":         {expPanic: true},
	}

	for testName, tc := range cases {
		t.Run(testName, func(t *testing.T) {
			if tc.expPanic {
				require.Panics(t, func() {
					PrefixRange(tc.src)
				})
				return
			}
			start, end := PrefixRange(tc.src)
			assert.Equal(t, tc.expStart, start)
			assert.Equal(t, tc.expEnd, end)
		})
	}
}
