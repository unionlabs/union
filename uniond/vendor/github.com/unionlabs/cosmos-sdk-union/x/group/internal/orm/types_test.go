package orm

import (
	"reflect"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/group/errors"
	"cosmossdk.io/x/group/internal/orm/prefixstore"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func TestTypeSafeRowGetter(t *testing.T) {
	key := storetypes.NewKVStoreKey("test")
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	prefixKey := [2]byte{0x2}
	store := prefixstore.New(runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx), prefixKey[:])

	md := testdata.TableModel{
		Id:   1,
		Name: "some name",
	}
	bz, err := md.Marshal()
	require.NoError(t, err)
	require.NoError(t, store.Set(EncodeSequence(1), bz))

	specs := map[string]struct {
		srcRowID     RowID
		srcModelType reflect.Type
		expObj       interface{}
		expErr       *errorsmod.Error
	}{
		"happy path": {
			srcRowID:     EncodeSequence(1),
			srcModelType: reflect.TypeOf(testdata.TableModel{}),
			expObj:       md,
		},
		"unknown rowID should return sdkerrors.ErrNotFound": {
			srcRowID:     EncodeSequence(2),
			srcModelType: reflect.TypeOf(testdata.TableModel{}),
			expErr:       sdkerrors.ErrNotFound,
		},
		"wrong type should cause sdkerrors.ErrInvalidType": {
			srcRowID:     EncodeSequence(1),
			srcModelType: reflect.TypeOf(testdata.Cat{}),
			expErr:       sdkerrors.ErrInvalidType,
		},
		"empty rowID not allowed": {
			srcRowID:     []byte{},
			srcModelType: reflect.TypeOf(testdata.TableModel{}),
			expErr:       errors.ErrORMEmptyKey,
		},
		"nil rowID not allowed": {
			srcModelType: reflect.TypeOf(testdata.TableModel{}),
			expErr:       errors.ErrORMEmptyKey,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			interfaceRegistry := types.NewInterfaceRegistry()
			cdc := codec.NewProtoCodec(interfaceRegistry)

			getter := NewTypeSafeRowGetter(prefixKey, spec.srcModelType, cdc)
			var loadedObj testdata.TableModel

			err := getter(runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx), spec.srcRowID, &loadedObj)
			if spec.expErr != nil {
				require.True(t, spec.expErr.Is(err), err)
				return
			}
			require.NoError(t, err)
			assert.Equal(t, spec.expObj, loadedObj)
		})
	}
}
