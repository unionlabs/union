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
)

func TestPrimaryKeyTablePrefixScan(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)
	ac := address.NewBech32Codec("cosmos")

	tb, err := NewPrimaryKeyTable(PrimaryKeyTablePrefix, &testdata.TableModel{}, cdc, ac)
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
		Metadata: metadata,
	}
	for _, g := range []testdata.TableModel{t1, t2, t3} {
		require.NoError(t, tb.Create(store, &g))
	}

	specs := map[string]struct {
		start, end []byte
		expResult  []testdata.TableModel
		expRowIDs  []RowID
		expError   *errorsmod.Error
		method     func(store corestore.KVStore, start, end []byte) (Iterator, error)
	}{
		"exact match with a single result": {
			start:     EncodeSequence(1), // == PrimaryKey(&t1)
			end:       EncodeSequence(2), // == PrimaryKey(&t2)
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1},
			expRowIDs: []RowID{PrimaryKey(&t1, ac)},
		},
		"one result by 1st byte": {
			start:     []byte{0},
			end:       EncodeSequence(2), // == PrimaryKey(&t2)
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1},
			expRowIDs: []RowID{PrimaryKey(&t1, ac)},
		},
		"open end query": {
			start:     EncodeSequence(3),
			end:       nil,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t3},
			expRowIDs: []RowID{PrimaryKey(&t3, ac)},
		},
		"open end query with all": {
			start:     EncodeSequence(1),
			end:       nil,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2, t3},
			expRowIDs: []RowID{PrimaryKey(&t1, ac), PrimaryKey(&t2, ac), PrimaryKey(&t3, ac)},
		},
		"open start query": {
			start:     nil,
			end:       EncodeSequence(3),
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2},
			expRowIDs: []RowID{PrimaryKey(&t1, ac), PrimaryKey(&t2, ac)},
		},
		"open start and end query": {
			start:     nil,
			end:       nil,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2, t3},
			expRowIDs: []RowID{PrimaryKey(&t1, ac), PrimaryKey(&t2, ac), PrimaryKey(&t3, ac)},
		},
		"all matching 1st byte": {
			start:     []byte{0},
			end:       nil,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2, t3},
			expRowIDs: []RowID{PrimaryKey(&t1, ac), PrimaryKey(&t2, ac), PrimaryKey(&t3, ac)},
		},
		"non matching 1st byte": {
			start:     []byte{1},
			end:       nil,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{},
		},
		"start equals end": {
			start:    EncodeSequence(1),
			end:      EncodeSequence(1),
			method:   tb.PrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"start after end": {
			start:    EncodeSequence(2),
			end:      EncodeSequence(1),
			method:   tb.PrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"reverse: exact match with a single result": {
			start:     EncodeSequence(1), // == PrimaryKey(&t1)
			end:       EncodeSequence(2), // == PrimaryKey(&t2)
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t1},
			expRowIDs: []RowID{PrimaryKey(&t1, ac)},
		},
		"reverse: one result by 1st byte": {
			start:     []byte{0},
			end:       EncodeSequence(2), // == PrimaryKey(&t2)
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t1},
			expRowIDs: []RowID{PrimaryKey(&t1, ac)},
		},
		"reverse: open end query": {
			start:     EncodeSequence(3),
			end:       nil,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3},
			expRowIDs: []RowID{PrimaryKey(&t3, ac)},
		},
		"reverse: open end query with all": {
			start:     EncodeSequence(1),
			end:       nil,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3, t2, t1},
			expRowIDs: []RowID{PrimaryKey(&t3, ac), PrimaryKey(&t2, ac), PrimaryKey(&t1, ac)},
		},
		"reverse: open start query": {
			start:     nil,
			end:       EncodeSequence(3),
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t2, t1},
			expRowIDs: []RowID{PrimaryKey(&t2, ac), PrimaryKey(&t1, ac)},
		},
		"reverse: open start and end query": {
			start:     nil,
			end:       nil,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3, t2, t1},
			expRowIDs: []RowID{PrimaryKey(&t3, ac), PrimaryKey(&t2, ac), PrimaryKey(&t1, ac)},
		},
		"reverse: all matching 1st byte": {
			start:     []byte{0},
			end:       nil,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3, t2, t1},
			expRowIDs: []RowID{PrimaryKey(&t3, ac), PrimaryKey(&t2, ac), PrimaryKey(&t1, ac)},
		},
		"reverse: non matching prefix": {
			start:     []byte{1},
			end:       nil,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{},
		},
		"reverse: start equals end": {
			start:    EncodeSequence(1),
			end:      EncodeSequence(1),
			method:   tb.ReversePrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"reverse: start after end": {
			start:    EncodeSequence(2),
			end:      EncodeSequence(1),
			method:   tb.ReversePrefixScan,
			expError: errors.ErrORMInvalidArgument,
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

func TestContains(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	key := storetypes.NewKVStoreKey("test")
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)

	tb, err := NewPrimaryKeyTable(PrimaryKeyTablePrefix, &testdata.TableModel{}, cdc, address.NewBech32Codec("cosmos"))
	require.NoError(t, err)

	obj := testdata.TableModel{
		Id:   1,
		Name: "Some name",
	}
	err = tb.Create(store, &obj)
	require.NoError(t, err)

	specs := map[string]struct {
		src PrimaryKeyed
		exp bool
	}{
		"same object": {src: &obj, exp: true},
		"clone": {
			src: &testdata.TableModel{
				Id:   1,
				Name: "Some name",
			},
			exp: true,
		},
		"different primary key": {
			src: &testdata.TableModel{
				Id:   2,
				Name: "Some name",
			},
			exp: false,
		},
		"different type, same key": {
			src: mockPrimaryKeyed{&obj},
			exp: false,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			got := tb.Contains(store, spec.src)
			assert.Equal(t, spec.exp, got)
		})
	}
}

type mockPrimaryKeyed struct {
	*testdata.TableModel
}
