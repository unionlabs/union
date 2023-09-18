package orm

import (
	"math"
	"testing"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestAutoUInt64PrefixScan(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	tb, err := NewAutoUInt64Table(AutoUInt64TablePrefix, AutoUInt64TableSeqPrefix, &testdata.TableModel{}, cdc)
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
		_, err := tb.Create(store, &g)
		require.NoError(t, err)
	}

	specs := map[string]struct {
		start, end uint64
		expResult  []testdata.TableModel
		expRowIDs  []RowID
		expError   *sdkerrors.Error
		method     func(store sdk.KVStore, start uint64, end uint64) (Iterator, error)
	}{
		"first element": {
			start:     1,
			end:       2,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1},
			expRowIDs: []RowID{EncodeSequence(1)},
		},
		"first 2 elements": {
			start:     1,
			end:       3,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2},
			expRowIDs: []RowID{EncodeSequence(1), EncodeSequence(2)},
		},
		"first 3 elements": {
			start:     1,
			end:       4,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2, t3},
			expRowIDs: []RowID{EncodeSequence(1), EncodeSequence(2), EncodeSequence(3)},
		},
		"search with max end": {
			start:     1,
			end:       math.MaxUint64,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t1, t2, t3},
			expRowIDs: []RowID{EncodeSequence(1), EncodeSequence(2), EncodeSequence(3)},
		},
		"2 to end": {
			start:     2,
			end:       5,
			method:    tb.PrefixScan,
			expResult: []testdata.TableModel{t2, t3},
			expRowIDs: []RowID{EncodeSequence(2), EncodeSequence(3)},
		},
		"start after end should fail": {
			start:    2,
			end:      1,
			method:   tb.PrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"start equals end should fail": {
			start:    1,
			end:      1,
			method:   tb.PrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"reverse first element": {
			start:     1,
			end:       2,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t1},
			expRowIDs: []RowID{EncodeSequence(1)},
		},
		"reverse first 2 elements": {
			start:     1,
			end:       3,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t2, t1},
			expRowIDs: []RowID{EncodeSequence(2), EncodeSequence(1)},
		},
		"reverse first 3 elements": {
			start:     1,
			end:       4,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3, t2, t1},
			expRowIDs: []RowID{EncodeSequence(3), EncodeSequence(2), EncodeSequence(1)},
		},
		"reverse search with max end": {
			start:     1,
			end:       math.MaxUint64,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3, t2, t1},
			expRowIDs: []RowID{EncodeSequence(3), EncodeSequence(2), EncodeSequence(1)},
		},
		"reverse 2 to end": {
			start:     2,
			end:       5,
			method:    tb.ReversePrefixScan,
			expResult: []testdata.TableModel{t3, t2},
			expRowIDs: []RowID{EncodeSequence(3), EncodeSequence(2)},
		},
		"reverse start after end should fail": {
			start:    2,
			end:      1,
			method:   tb.ReversePrefixScan,
			expError: errors.ErrORMInvalidArgument,
		},
		"reverse start equals end should fail": {
			start:    1,
			end:      1,
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
