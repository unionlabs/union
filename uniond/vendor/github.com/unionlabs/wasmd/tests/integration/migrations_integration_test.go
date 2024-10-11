package integration

import (
	"testing"

	"cosmossdk.io/math/unsafe"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	upgradetypes "cosmossdk.io/x/upgrade/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/module"

	"github.com/CosmWasm/wasmd/app"
	v2 "github.com/CosmWasm/wasmd/x/wasm/migrations/v2"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestModuleMigrations(t *testing.T) {
	wasmApp := app.Setup(t)
	myAddress := sdk.AccAddress(rand.Bytes(address.Len))

	upgradeHandler := func(ctx sdk.Context, _ upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {
		return wasmApp.ModuleManager.RunMigrations(ctx, wasmApp.Configurator(), fromVM)
	}

	specs := map[string]struct {
		setup        func(ctx sdk.Context)
		startVersion uint64
		exp          types.Params
	}{
		"with legacy params migrated": {
			startVersion: 1,
			setup: func(ctx sdk.Context) {
				params := v2.Params{
					CodeUploadAccess:             v2.AccessConfig{Permission: v2.AccessTypeNobody},
					InstantiateDefaultPermission: v2.AccessTypeNobody,
				}

				// upgrade code shipped with v0.40
				// https://github.com/CosmWasm/wasmd/blob/v0.40.0/app/upgrades.go#L66
				sp, _ := wasmApp.ParamsKeeper.GetSubspace(types.ModuleName)
				keyTable := v2.ParamKeyTable()
				if !sp.HasKeyTable() {
					sp.WithKeyTable(keyTable)
				}

				sp.SetParamSet(ctx, &params)
			},
			exp: types.Params{
				CodeUploadAccess:             types.AllowNobody,
				InstantiateDefaultPermission: types.AccessTypeNobody,
			},
		},
		"with legacy one address type replaced": {
			startVersion: 1,
			setup: func(ctx sdk.Context) {
				params := v2.Params{
					CodeUploadAccess:             v2.AccessConfig{Permission: v2.AccessTypeOnlyAddress, Address: myAddress.String()},
					InstantiateDefaultPermission: v2.AccessTypeNobody,
				}

				// upgrade code shipped with v0.40
				// https://github.com/CosmWasm/wasmd/blob/v0.40.0/app/upgrades.go#L66
				sp, _ := wasmApp.ParamsKeeper.GetSubspace(types.ModuleName)
				keyTable := v2.ParamKeyTable()
				if !sp.HasKeyTable() {
					sp.WithKeyTable(keyTable)
				}

				sp.SetParamSet(ctx, &params)
			},
			exp: types.Params{
				CodeUploadAccess:             types.AccessTypeAnyOfAddresses.With(myAddress),
				InstantiateDefaultPermission: types.AccessTypeNobody,
			},
		},
		"fresh from genesis": {
			startVersion: wasmApp.ModuleManager.GetVersionMap()[types.ModuleName], // latest
			setup:        func(ctx sdk.Context) {},
			exp:          types.DefaultParams(),
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ := wasmApp.BaseApp.NewContext(false).CacheContext()
			spec.setup(ctx)

			fromVM, err := wasmApp.UpgradeKeeper.GetModuleVersionMap(ctx)
			require.NoError(t, err)
			fromVM[types.ModuleName] = spec.startVersion
			_, err = upgradeHandler(ctx, upgradetypes.Plan{Name: "testing"}, fromVM)
			require.NoError(t, err)

			// when
			gotVM, err := wasmApp.ModuleManager.RunMigrations(ctx, wasmApp.Configurator(), fromVM)

			// then
			require.NoError(t, err)
			var expModuleVersion uint64 = 4
			assert.Equal(t, expModuleVersion, gotVM[types.ModuleName])
			gotParams := wasmApp.WasmKeeper.GetParams(ctx)
			assert.Equal(t, spec.exp, gotParams)
		})
	}
}

func TestAccessConfigMigrations(t *testing.T) {
	addr := "cosmos1vx8knpllrj7n963p9ttd80w47kpacrhuts497x"
	address, err := sdk.AccAddressFromBech32(addr)
	require.NoError(t, err)

	wasmApp := app.Setup(t)

	upgradeHandler := func(ctx sdk.Context, _ upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {
		return wasmApp.ModuleManager.RunMigrations(ctx, wasmApp.Configurator(), fromVM)
	}

	ctx, _ := wasmApp.BaseApp.NewContext(false).CacheContext()

	// any address permission
	code1, err := storeCode(ctx, wasmApp, types.AccessTypeAnyOfAddresses.With(address))
	require.NoError(t, err)

	// allow everybody permission
	code2, err := storeCode(ctx, wasmApp, types.AllowEverybody)
	require.NoError(t, err)

	// allow nobody permission
	code3, err := storeCode(ctx, wasmApp, types.AllowNobody)
	require.NoError(t, err)

	fromVM, err := wasmApp.UpgradeKeeper.GetModuleVersionMap(ctx)
	require.NoError(t, err)
	fromVM[types.ModuleName] = wasmApp.ModuleManager.GetVersionMap()[types.ModuleName]
	_, err = upgradeHandler(ctx, upgradetypes.Plan{Name: "testing"}, fromVM)
	require.NoError(t, err)

	// when
	gotVM, err := wasmApp.ModuleManager.RunMigrations(ctx, wasmApp.Configurator(), fromVM)

	// then
	require.NoError(t, err)
	var expModuleVersion uint64 = 4
	assert.Equal(t, expModuleVersion, gotVM[types.ModuleName])

	// any address was not migrated
	assert.Equal(t, types.AccessTypeAnyOfAddresses.With(address), wasmApp.WasmKeeper.GetCodeInfo(ctx, code1).InstantiateConfig)

	// allow everybody was not migrated
	assert.Equal(t, types.AllowEverybody, wasmApp.WasmKeeper.GetCodeInfo(ctx, code2).InstantiateConfig)

	// allow nodoby was not migrated
	assert.Equal(t, types.AllowNobody, wasmApp.WasmKeeper.GetCodeInfo(ctx, code3).InstantiateConfig)
}

func storeCode(ctx sdk.Context, wasmApp *app.WasmApp, instantiatePermission types.AccessConfig) (codeID uint64, err error) {
	msg := types.MsgStoreCodeFixture(func(m *types.MsgStoreCode) {
		m.WASMByteCode = wasmContract
		m.InstantiatePermission = &instantiatePermission
	})
	rsp, err := wasmApp.MsgServiceRouter().Handler(msg)(ctx, msg)
	if err != nil {
		return
	}

	var result types.MsgStoreCodeResponse
	err = wasmApp.AppCodec().Unmarshal(rsp.Data, &result)
	if err != nil {
		return
	}

	codeID = result.CodeID
	return
}
