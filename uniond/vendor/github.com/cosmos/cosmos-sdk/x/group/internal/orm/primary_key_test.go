package orm

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

func TestPrimaryKeyTablePrefixScan(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	tb, err := NewPrimaryKeyTable(PrimaryKeyTablePrefix, &testdata.TableModel{}, cdc)
	require.NoError(t, err)

	ctx := NewMockContext()
	store := ctx.KVStore(sdk.NewKVStoreKey("test"))

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
		expError   *sdkerrors.Error
		method     func(store sdk.KVStore, start, end []byte) (Iterator, error)
	}{
		"exact match with a single result": {
			start:     EncodeSequence(1), // == PrimaryKey(&t1)
			end:       EncodeSequence(2), // == PrimaryKey(&t2)
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1},
			expRowIDs: []RowID{PrimaryKey(&t1)},
		},
		"one result by 1st byte": {
			start:     []byte{0},
			end:       EncodeSequence(2), // == PrimaryKey(&t2)
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1},
			expRowIDs: []RowID{PrimaryKey(&t1)},
		},
		"open end query": {
			start:     EncodeSequence(3),
			end:       nil,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t3},
			expRowIDs: []RowID{PrimaryKey(&t3)},
		},
		"open end query with all": {
			start:     EncodeSequence(1),
			end:       nil,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2, t3},
			expRowIDs: []RowID{PrimaryKey(&t1), PrimaryKey(&t2), PrimaryKey(&t3)},
		},
		"open start query": {
			start:     nil,
			end:       EncodeSequence(3),
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2},
			expRowIDs: []RowID{PrimaryKey(&t1), PrimaryKey(&t2)},
		},
		"open start and end query": {
			start:     nil,
			end:       nil,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2, t3},
			expRowIDs: []RowID{PrimaryKey(&t1), PrimaryKey(&t2), PrimaryKey(&t3)},
		},
		"all matching 1st byte": {
			start:     []byte{0},
			end:       nil,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2, t3},
			expRowIDs: []RowID{PrimaryKey(&t1), PrimaryKey(&t2), PrimaryKey(&t3)},
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
			expRowIDs: []RowID{PrimaryKey(&t1)},
		},
		"reverse: one result by 1st byte": {
			start:     []byte{0},
			end:       EncodeSequence(2), // == PrimaryKey(&t2)
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t1},
			expRowIDs: []RowID{PrimaryKey(&t1)},
		},
		"reverse: open end query": {
			start:     EncodeSequence(3),
			end:       nil,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3},
			expRowIDs: []RowID{PrimaryKey(&t3)},
		},
		"reverse: open end query with all": {
			start:     EncodeSequence(1),
			end:       nil,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3, t2, t1},
			expRowIDs: []RowID{PrimaryKey(&t3), PrimaryKey(&t2), PrimaryKey(&t1)},
		},
		"reverse: open start query": {
			start:     nil,
			end:       EncodeSequence(3),
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t2, t1},
			expRowIDs: []RowID{PrimaryKey(&t2), PrimaryKey(&t1)},
		},
		"reverse: open start and end query": {
			start:     nil,
			end:       nil,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3, t2, t1},
			expRowIDs: []RowID{PrimaryKey(&t3), PrimaryKey(&t2), PrimaryKey(&t1)},
		},
		"reverse: all matching 1st byte": {
			start:     []byte{0},
			end:       nil,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3, t2, t1},
			expRowIDs: []RowID{PrimaryKey(&t3), PrimaryKey(&t2), PrimaryKey(&t1)},
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

	ctx := NewMockContext()
	store := ctx.KVStore(sdk.NewKVStoreKey("test"))

	tb, err := NewPrimaryKeyTable(PrimaryKeyTablePrefix, &testdata.TableModel{}, cdc)
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
