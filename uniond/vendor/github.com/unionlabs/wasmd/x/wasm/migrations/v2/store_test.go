package v2_test

import (
	"testing"

	"cosmossdk.io/math/unsafe"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	paramskeeper "cosmossdk.io/x/params/keeper"
	paramstypes "cosmossdk.io/x/params/types"

	"github.com/CosmWasm/wasmd/x/wasm"
	v2 "github.com/CosmWasm/wasmd/x/wasm/migrations/v2"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestMigrate(t *testing.T) {
	cfg := moduletestutil.MakeTestEncodingConfig(wasm.AppModuleBasic{})
	cdc := cfg.Codec
	var (
		wasmStoreKey    = storetypes.NewKVStoreKey(types.StoreKey)
		paramsStoreKey  = storetypes.NewKVStoreKey(paramstypes.StoreKey)
		paramsTStoreKey = storetypes.NewTransientStoreKey(paramstypes.TStoreKey)
		myAddress       = sdk.AccAddress(rand.Bytes(address.Len))
	)
	specs := map[string]struct {
		src v2.Params
	}{
		"one address": {
			src: v2.Params{
				CodeUploadAccess: v2.AccessConfig{
					Permission: v2.AccessTypeOnlyAddress,
					Address:    myAddress.String(),
				},
				InstantiateDefaultPermission: v2.AccessTypeNobody,
			},
		},
		"multiple addresses": {
			src: v2.Params{
				CodeUploadAccess: v2.AccessConfig{
					Permission: v2.AccessTypeAnyOfAddresses,
					Addresses:  []string{myAddress.String(), sdk.AccAddress(rand.Bytes(address.Len)).String()},
				},
				InstantiateDefaultPermission: v2.AccessTypeEverybody,
			},
		},
		"everybody": {
			src: v2.Params{
				CodeUploadAccess: v2.AccessConfig{
					Permission: v2.AccessTypeEverybody,
				},
				InstantiateDefaultPermission: v2.AccessTypeEverybody,
			},
		},
		"nobody": {
			src: v2.Params{
				CodeUploadAccess: v2.AccessConfig{
					Permission: v2.AccessTypeNobody,
				},
				InstantiateDefaultPermission: v2.AccessTypeNobody,
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			paramsKeeper := paramskeeper.NewKeeper(cdc, cfg.Amino, paramsStoreKey, paramsTStoreKey)
			ctx := testutil.DefaultContextWithKeys(
				map[string]*storetypes.KVStoreKey{
					paramstypes.StoreKey: paramsStoreKey,
					types.StoreKey:       wasmStoreKey,
				},
				map[string]*storetypes.TransientStoreKey{
					paramstypes.TStoreKey: paramsTStoreKey,
				},
				nil,
			)

			// register legacy parameters
			params := spec.src
			subspace := paramsKeeper.Subspace(types.ModuleName)
			subspace.WithKeyTable(v2.ParamKeyTable())
			subspace.SetParamSet(ctx, &params)

			// when
			require.NoError(t, v2.MigrateStore(ctx, runtime.NewKVStoreService(wasmStoreKey), subspace, cdc))

			var res v2.Params
			bz := ctx.KVStore(wasmStoreKey).Get(types.ParamsKey)
			require.NoError(t, cdc.Unmarshal(bz, &res))
			assert.Equal(t, params, res)
		})
	}
}
