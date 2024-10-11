package orm

import (
	"testing"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"
	"pgregory.net/rapid"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/group/errors"

	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/types/query"
)

func TestPaginationProperty(t *testing.T) {
	t.Run("TestPagination", rapid.MakeCheck(func(t *rapid.T) {
		// Create a slice of group members
		tableModels := rapid.SliceOf(genTableModel).Draw(t, "tableModels")

		// Choose a random limit for paging
		upperLimit := uint64(len(tableModels))
		if upperLimit == 0 {
			upperLimit = 1
		}
		limit := rapid.Uint64Range(1, upperLimit).Draw(t, "limit")

		// Reconstruct the slice from offset pages
		reconstructedTableModels := make([]*testdata.TableModel, 0, len(tableModels))
		for offset := uint64(0); offset < uint64(len(tableModels)); offset += limit {
			pageRequest := &query.PageRequest{
				Key:        nil,
				Offset:     offset,
				Limit:      limit,
				CountTotal: false,
				Reverse:    false,
			}
			end := offset + limit
			if end > uint64(len(tableModels)) {
				end = uint64(len(tableModels))
			}
			dest := reconstructedTableModels[offset:end]
			tableModelsIt := testTableModelIterator(tableModels, nil)
			_, err := Paginate(tableModelsIt, pageRequest, &dest)
			require.NoError(t, err)
			reconstructedTableModels = append(reconstructedTableModels, dest...)
		}

		// Should be the same slice
		require.Equal(t, len(tableModels), len(reconstructedTableModels))
		for i, gm := range tableModels {
			require.Equal(t, *gm, *reconstructedTableModels[i])
		}

		// Reconstruct the slice from keyed pages
		reconstructedTableModels = make([]*testdata.TableModel, 0, len(tableModels))
		var start uint64
		key := EncodeSequence(0)
		for key != nil {
			pageRequest := &query.PageRequest{
				Key:        key,
				Offset:     0,
				Limit:      limit,
				CountTotal: false,
				Reverse:    false,
			}

			end := start + limit
			if end > uint64(len(tableModels)) {
				end = uint64(len(tableModels))
			}

			dest := reconstructedTableModels[start:end]
			tableModelsIt := testTableModelIterator(tableModels, key)

			resp, err := Paginate(tableModelsIt, pageRequest, &dest)
			require.NoError(t, err)
			key = resp.NextKey

			reconstructedTableModels = append(reconstructedTableModels, dest...)

			start += limit
		}

		// Should be the same slice
		require.Equal(t, len(tableModels), len(reconstructedTableModels))
		for i, gm := range tableModels {
			require.Equal(t, *gm, *reconstructedTableModels[i])
		}
	}))
}

func testTableModelIterator(tms []*testdata.TableModel, key RowID) Iterator {
	var closed bool
	var index int
	if key != nil {
		index = int(DecodeSequence(key))
	}
	return IteratorFunc(func(dest proto.Message) (RowID, error) {
		if dest == nil {
			return nil, errorsmod.Wrap(errors.ErrORMInvalidArgument, "destination object must not be nil")
		}

		if index == len(tms) {
			closed = true
		}

		if closed {
			return nil, errors.ErrORMIteratorDone
		}

		rowID := EncodeSequence(uint64(index))

		bytes, err := tms[index].Marshal()
		if err != nil {
			return nil, err
		}

		index++

		return rowID, proto.Unmarshal(bytes, dest)
	})
}
