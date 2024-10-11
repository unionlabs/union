package orm

import (
	"testing"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/group/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

func TestReadAll(t *testing.T) {
	specs := map[string]struct {
		srcIT     Iterator
		destSlice func() ModelSlicePtr
		expErr    *errorsmod.Error
		expIDs    []RowID
		expResult ModelSlicePtr
	}{
		"all good with object slice": {
			srcIT: mockIter(EncodeSequence(1), &testdata.TableModel{Name: "test"}),
			destSlice: func() ModelSlicePtr {
				x := make([]testdata.TableModel, 1)
				return &x
			},
			expIDs:    []RowID{EncodeSequence(1)},
			expResult: &[]testdata.TableModel{{Name: "test"}},
		},
		"all good with pointer slice": {
			srcIT: mockIter(EncodeSequence(1), &testdata.TableModel{Name: "test"}),
			destSlice: func() ModelSlicePtr {
				x := make([]*testdata.TableModel, 1)
				return &x
			},
			expIDs:    []RowID{EncodeSequence(1)},
			expResult: &[]*testdata.TableModel{{Name: "test"}},
		},
		"dest slice empty": {
			srcIT: mockIter(EncodeSequence(1), &testdata.TableModel{}),
			destSlice: func() ModelSlicePtr {
				x := make([]testdata.TableModel, 0)
				return &x
			},
			expIDs:    []RowID{EncodeSequence(1)},
			expResult: &[]testdata.TableModel{{}},
		},
		"dest pointer with nil value": {
			srcIT: mockIter(EncodeSequence(1), &testdata.TableModel{}),
			destSlice: func() ModelSlicePtr {
				return (*[]testdata.TableModel)(nil)
			},
			expErr: errors.ErrORMInvalidArgument,
		},
		"iterator is nil": {
			srcIT:     nil,
			destSlice: func() ModelSlicePtr { return new([]testdata.TableModel) },
			expErr:    errors.ErrORMInvalidArgument,
		},
		"dest slice is nil": {
			srcIT:     noopIter(),
			destSlice: func() ModelSlicePtr { return nil },
			expErr:    errors.ErrORMInvalidArgument,
		},
		"dest slice is not a pointer": {
			srcIT:     IteratorFunc(nil),
			destSlice: func() ModelSlicePtr { return make([]testdata.TableModel, 1) },
			expErr:    errors.ErrORMInvalidArgument,
		},
		"error on loadNext is returned": {
			srcIT: NewInvalidIterator(),
			destSlice: func() ModelSlicePtr {
				x := make([]testdata.TableModel, 1)
				return &x
			},
			expErr: errors.ErrORMInvalidIterator,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			loaded := spec.destSlice()
			ids, err := ReadAll(spec.srcIT, loaded)
			require.True(t, spec.expErr.Is(err), "expected %s but got %s", spec.expErr, err)
			assert.Equal(t, spec.expIDs, ids)
			if err == nil {
				assert.Equal(t, spec.expResult, loaded)
			}
		})
	}
}

func TestLimitedIterator(t *testing.T) {
	specs := map[string]struct {
		parent      Iterator
		max         int
		expectErr   bool
		expectedErr string
		exp         []testdata.TableModel
	}{
		"nil parent": {
			parent:      nil,
			max:         0,
			expectErr:   true,
			expectedErr: "parent iterator must not be nil",
		},
		"negative max": {
			parent:      mockIter(EncodeSequence(1), &testdata.TableModel{Name: "test"}),
			max:         -1,
			expectErr:   true,
			expectedErr: "quantity must not be negative",
		},
		"all from range with max > length": {
			parent: mockIter(EncodeSequence(1), &testdata.TableModel{Name: "test"}),
			max:    2,
			exp:    []testdata.TableModel{{Name: "test"}},
		},
		"up to max": {
			parent: mockIter(EncodeSequence(1), &testdata.TableModel{Name: "test"}),
			max:    1,
			exp:    []testdata.TableModel{{Name: "test"}},
		},
		"none when max = 0": {
			parent: mockIter(EncodeSequence(1), &testdata.TableModel{Name: "test"}),
			max:    0,
			exp:    []testdata.TableModel{},
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			src, err := LimitIterator(spec.parent, spec.max)
			if spec.expectErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), spec.expectedErr)
			} else {
				require.NoError(t, err)
				var loaded []testdata.TableModel
				_, err := ReadAll(src, &loaded)
				require.NoError(t, err)
				assert.EqualValues(t, spec.exp, loaded)
			}
		})
	}
}

func TestFirst(t *testing.T) {
	testCases := []struct {
		name          string
		iterator      Iterator
		dest          proto.Message
		expectErr     bool
		expectedErr   string
		expectedRowID RowID
		expectedDest  proto.Message
	}{
		{
			name:        "nil iterator",
			iterator:    nil,
			dest:        &testdata.TableModel{},
			expectErr:   true,
			expectedErr: "iterator must not be nil",
		},
		{
			name:        "nil dest",
			iterator:    mockIter(EncodeSequence(1), &testdata.TableModel{Name: "test"}),
			dest:        nil,
			expectErr:   true,
			expectedErr: "destination object must not be nil",
		},
		{
			name:          "all not nil",
			iterator:      mockIter(EncodeSequence(1), &testdata.TableModel{Name: "test"}),
			dest:          &testdata.TableModel{},
			expectErr:     false,
			expectedRowID: EncodeSequence(1),
			expectedDest:  &testdata.TableModel{Name: "test"},
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			rowID, err := First(tc.iterator, tc.dest)
			if tc.expectErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expectedErr)
			} else {
				require.NoError(t, err)
				require.Equal(t, tc.expectedRowID, rowID)
				require.Equal(t, tc.expectedDest, tc.dest)
			}
		})
	}
}

func TestPaginate(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	tb, err := NewAutoUInt64Table(AutoUInt64TablePrefix, AutoUInt64TableSeqPrefix, &testdata.TableModel{}, cdc, address.NewBech32Codec("cosmos"))
	require.NoError(t, err)
	idx, err := NewIndex(tb, AutoUInt64TableModelByMetadataPrefix, func(val interface{}) ([]interface{}, error) {
		return []interface{}{val.(*testdata.TableModel).Metadata}, nil
	}, testdata.TableModel{}.Metadata)
	require.NoError(t, err)

	key := storetypes.NewKVStoreKey("test")
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)

	metadata := []byte("metadata")
	t1 := testdata.TableModel{
		Id:       1,
		Name:     "my test 1",
		Metadata: metadata,
	}
	t2 := testdata.TableModel{
		Id:       2,
		Name:     "my test 2",
		Metadata: metadata,
	}
	t3 := testdata.TableModel{
		Id:       3,
		Name:     "my test 3",
		Metadata: []byte("other-metadata"),
	}
	t4 := testdata.TableModel{
		Id:       4,
		Name:     "my test 4",
		Metadata: metadata,
	}
	t5 := testdata.TableModel{
		Id:       5,
		Name:     "my test 5",
		Metadata: []byte("other-metadata"),
	}

	for _, g := range []testdata.TableModel{t1, t2, t3, t4, t5} {
		_, err := tb.Create(store, &g)
		require.NoError(t, err)
	}

	specs := map[string]struct {
		pageReq    *query.PageRequest
		expPageRes *query.PageResponse
		exp        []testdata.TableModel
		key        []byte
		expErr     bool
	}{
		"one item": {
			pageReq:    &query.PageRequest{Key: nil, Limit: 1},
			exp:        []testdata.TableModel{t1},
			expPageRes: &query.PageResponse{Total: 0, NextKey: EncodeSequence(2)},
			key:        metadata,
		},
		"with both key and offset": {
			pageReq: &query.PageRequest{Key: EncodeSequence(2), Offset: 1},
			expErr:  true,
			key:     metadata,
		},
		"up to max": {
			pageReq:    &query.PageRequest{Key: nil, Limit: 3, CountTotal: true},
			exp:        []testdata.TableModel{t1, t2, t4},
			expPageRes: &query.PageResponse{Total: 3, NextKey: nil},
			key:        metadata,
		},
		"no results": {
			pageReq:    &query.PageRequest{Key: nil, Limit: 2, CountTotal: true},
			exp:        []testdata.TableModel{},
			expPageRes: &query.PageResponse{Total: 0, NextKey: nil},
			key:        sdk.AccAddress([]byte("no-group-address")),
		},
		"with offset and count total": {
			pageReq:    &query.PageRequest{Key: nil, Offset: 1, Limit: 2, CountTotal: true},
			exp:        []testdata.TableModel{t2, t4},
			expPageRes: &query.PageResponse{Total: 3, NextKey: nil},
			key:        metadata,
		},
		"nil/default page req (limit = 100 > number of items)": {
			pageReq:    nil,
			exp:        []testdata.TableModel{t1, t2, t4},
			expPageRes: &query.PageResponse{Total: 3, NextKey: nil},
			key:        metadata,
		},
		"with key and limit < number of elem (count total is ignored in this case)": {
			pageReq:    &query.PageRequest{Key: EncodeSequence(2), Limit: 1, CountTotal: true},
			exp:        []testdata.TableModel{t2},
			expPageRes: &query.PageResponse{Total: 0, NextKey: EncodeSequence(4)},
			key:        metadata,
		},
		"with key and limit >= number of elem": {
			pageReq:    &query.PageRequest{Key: EncodeSequence(2), Limit: 2},
			exp:        []testdata.TableModel{t2, t4},
			expPageRes: &query.PageResponse{Total: 0, NextKey: nil},
			key:        metadata,
		},
		"with nothing left to iterate from key": {
			pageReq:    &query.PageRequest{Key: EncodeSequence(5)},
			exp:        []testdata.TableModel{},
			expPageRes: &query.PageResponse{Total: 0, NextKey: nil},
			key:        metadata,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			var loaded []testdata.TableModel

			it, err := idx.GetPaginated(store, spec.key, spec.pageReq)
			require.NoError(t, err)

			res, err := Paginate(it, spec.pageReq, &loaded)
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				assert.EqualValues(t, spec.exp, loaded)
				assert.EqualValues(t, spec.expPageRes.Total, res.Total)
				assert.EqualValues(t, spec.expPageRes.NextKey, res.NextKey)
			}
		})
	}

	t.Run("nil iterator", func(t *testing.T) {
		var loaded []testdata.TableModel
		res, err := Paginate(nil, &query.PageRequest{}, &loaded)
		require.Error(t, err)
		require.Contains(t, err.Error(), "iterator must not be nil")
		require.Nil(t, res)
	})

	t.Run("non-slice destination", func(t *testing.T) {
		var loaded testdata.TableModel
		res, err := Paginate(
			mockIter(EncodeSequence(1), &testdata.TableModel{Name: "test"}),
			&query.PageRequest{},
			&loaded,
		)
		require.Error(t, err)
		require.Contains(t, err.Error(), "destination must point to a slice")
		require.Nil(t, res)
	})
}

// mockIter encodes + decodes value object.
func mockIter(rowID RowID, val proto.Message) Iterator {
	b, err := proto.Marshal(val)
	if err != nil {
		panic(err)
	}
	return NewSingleValueIterator(rowID, b)
}

func noopIter() Iterator {
	return IteratorFunc(func(dest proto.Message) (RowID, error) {
		return nil, nil
	})
}
