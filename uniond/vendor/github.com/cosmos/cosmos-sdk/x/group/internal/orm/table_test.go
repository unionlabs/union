package orm

import (
	"fmt"
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

func TestNewTable(t *testing.T) {
	interfaceRegistry := types.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)

	testCases := []struct {
		name        string
		model       codec.ProtoMarshaler
		expectErr   bool
		expectedErr string
	}{
		{
			name:        "nil model",
			model:       nil,
			expectErr:   true,
			expectedErr: "Model must not be nil",
		},
		{
			name:      "all not nil",
			model:     &testdata.TableModel{},
			expectErr: false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			table, err := newTable([2]byte{0x1}, tc.model, cdc)
			if tc.expectErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expectedErr)
			} else {
				require.NoError(t, err)
				require.NotNil(t, table)
			}
		})
	}
}

func TestCreate(t *testing.T) {
	specs := map[string]struct {
		rowID  RowID
		src    codec.ProtoMarshaler
		expErr *sdkerrors.Error
	}{
		"empty rowID": {
			rowID: []byte{},
			src: &testdata.TableModel{
				Id:   1,
				Name: "some name",
			},
			expErr: errors.ErrORMEmptyKey,
		},
		"happy path": {
			rowID: EncodeSequence(1),
			src: &testdata.TableModel{
				Id:   1,
				Name: "some name",
			},
		},
		"wrong type": {
			rowID: EncodeSequence(1),
			src: &testdata.Cat{
				Moniker: "cat moniker",
				Lives:   10,
			},
			expErr: sdkerrors.ErrInvalidType,
		},
		"model validation fails": {
			rowID: EncodeSequence(1),
			src: &testdata.TableModel{
				Id:   1,
				Name: "",
			},
			expErr: testdata.ErrTest,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			interfaceRegistry := types.NewInterfaceRegistry()
			cdc := codec.NewProtoCodec(interfaceRegistry)

			ctx := NewMockContext()
			store := ctx.KVStore(sdk.NewKVStoreKey("test"))

			anyPrefix := [2]byte{0x10}
			myTable, err := newTable(anyPrefix, &testdata.TableModel{}, cdc)
			require.NoError(t, err)

			err = myTable.Create(store, spec.rowID, spec.src)

			require.True(t, spec.expErr.Is(err), err)
			shouldExists := spec.expErr == nil
			assert.Equal(t, shouldExists, myTable.Has(store, spec.rowID), fmt.Sprintf("expected %v", shouldExists))

			// then
			var loaded testdata.TableModel
			err = myTable.GetOne(store, spec.rowID, &loaded)
			if spec.expErr != nil {
				require.True(t, sdkerrors.ErrNotFound.Is(err))
				return
			}
			require.NoError(t, err)
			assert.Equal(t, spec.src, &loaded)
		})
	}
}

func TestUpdate(t *testing.T) {
	specs := map[string]struct {
		src    codec.ProtoMarshaler
		expErr *sdkerrors.Error
	}{
		"happy path": {
			src: &testdata.TableModel{
				Id:   1,
				Name: "some name",
			},
		},
		"wrong type": {
			src: &testdata.Cat{
				Moniker: "cat moniker",
				Lives:   10,
			},
			expErr: sdkerrors.ErrInvalidType,
		},
		"model validation fails": {
			src: &testdata.TableModel{
				Id:   1,
				Name: "",
			},
			expErr: testdata.ErrTest,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			interfaceRegistry := types.NewInterfaceRegistry()
			cdc := codec.NewProtoCodec(interfaceRegistry)

			ctx := NewMockContext()
			store := ctx.KVStore(sdk.NewKVStoreKey("test"))

			anyPrefix := [2]byte{0x10}
			myTable, err := newTable(anyPrefix, &testdata.TableModel{}, cdc)
			require.NoError(t, err)

			initValue := testdata.TableModel{
				Id:   1,
				Name: "old name",
			}

			err = myTable.Create(store, EncodeSequence(1), &initValue)
			require.NoError(t, err)

			// when
			err = myTable.Update(store, EncodeSequence(1), spec.src)
			require.True(t, spec.expErr.Is(err), "got ", err)

			// then
			var loaded testdata.TableModel
			require.NoError(t, myTable.GetOne(store, EncodeSequence(1), &loaded))
			if spec.expErr == nil {
				assert.Equal(t, spec.src, &loaded)
			} else {
				assert.Equal(t, initValue, loaded)
			}
		})
	}
}

func TestDelete(t *testing.T) {
	specs := map[string]struct {
		rowId  []byte
		expErr *sdkerrors.Error
	}{
		"happy path": {
			rowId: EncodeSequence(1),
		},
		"not found": {
			rowId:  []byte("not-found"),
			expErr: sdkerrors.ErrNotFound,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			interfaceRegistry := types.NewInterfaceRegistry()
			cdc := codec.NewProtoCodec(interfaceRegistry)

			ctx := NewMockContext()
			store := ctx.KVStore(sdk.NewKVStoreKey("test"))

			anyPrefix := [2]byte{0x10}
			myTable, err := newTable(anyPrefix, &testdata.TableModel{}, cdc)
			require.NoError(t, err)

			initValue := testdata.TableModel{
				Id:   1,
				Name: "some name",
			}

			err = myTable.Create(store, EncodeSequence(1), &initValue)
			require.NoError(t, err)

			// when
			err = myTable.Delete(store, spec.rowId)
			require.True(t, spec.expErr.Is(err), "got ", err)

			// then
			var loaded testdata.TableModel
			if spec.expErr == sdkerrors.ErrNotFound {
				require.NoError(t, myTable.GetOne(store, EncodeSequence(1), &loaded))
				assert.Equal(t, initValue, loaded)
			} else {
				err := myTable.GetOne(store, EncodeSequence(1), &loaded)
				require.Error(t, err)
				require.Equal(t, err, sdkerrors.ErrNotFound)
			}
		})
	}
}
