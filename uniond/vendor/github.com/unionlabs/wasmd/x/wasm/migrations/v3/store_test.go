package v3_test

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"

	"github.com/CosmWasm/wasmd/x/wasm"
	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/wasmtesting"
	v3 "github.com/CosmWasm/wasmd/x/wasm/migrations/v3"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestMigrate3To4(t *testing.T) {
	AvailableCapabilities := []string{"iterator", "staking", "stargate", "cosmwasm_1_1"}
	ctx, keepers := keeper.CreateTestInput(t, false, AvailableCapabilities)
	store := ctx.KVStore(keepers.WasmStoreKey)
	cdc := moduletestutil.MakeTestEncodingConfig(wasm.AppModuleBasic{}).Codec
	wasmKeeper := keepers.WasmKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := sdk.AccAddress(bytes.Repeat([]byte{1}, address.Len))
	keepers.Faucet.Fund(ctx, creator, deposit...)

	var mock wasmtesting.MockWasmEngine
	wasmtesting.MakeInstantiable(&mock)

	// contract with only address permission
	onlyAddrPermission := v3.AccessConfig{
		Permission: v3.AccessTypeOnlyAddress,
		Address:    creator.String(),
	}
	contract1 := keeper.StoreRandomContract(t, ctx, keepers, &mock)
	newInfo1 := wasmKeeper.GetCodeInfo(ctx, contract1.CodeID)
	legacyInfo1 := v3.CodeInfo{
		CodeHash:          newInfo1.CodeHash,
		Creator:           newInfo1.Creator,
		InstantiateConfig: onlyAddrPermission,
	}

	store.Set(types.GetCodeKey(contract1.CodeID), cdc.MustMarshal(&legacyInfo1))

	// contract with any addresses permission
	anyAddrPermission := v3.AccessConfig{
		Permission: v3.AccessTypeAnyOfAddresses,
		Addresses:  []string{creator.String()},
	}
	contract2 := keeper.StoreRandomContract(t, ctx, keepers, &mock)
	newInfo2 := wasmKeeper.GetCodeInfo(ctx, contract2.CodeID)
	legacyInfo2 := v3.CodeInfo{
		CodeHash:          newInfo2.CodeHash,
		Creator:           newInfo2.Creator,
		InstantiateConfig: anyAddrPermission,
	}

	store.Set(types.GetCodeKey(contract2.CodeID), cdc.MustMarshal(&legacyInfo2))

	// contract with everybody permission
	allowEverybodyPermission := v3.AccessConfig{
		Permission: v3.AccessTypeEverybody,
	}
	contract3 := keeper.StoreRandomContract(t, ctx, keepers, &mock)
	newInfo3 := wasmKeeper.GetCodeInfo(ctx, contract3.CodeID)
	legacyInfo3 := v3.CodeInfo{
		CodeHash:          newInfo3.CodeHash,
		Creator:           newInfo3.Creator,
		InstantiateConfig: allowEverybodyPermission,
	}

	store.Set(types.GetCodeKey(contract3.CodeID), cdc.MustMarshal(&legacyInfo3))

	// contract with nobody permission
	allowNobodyPermission := v3.AccessConfig{
		Permission: v3.AccessTypeNobody,
	}
	contract4 := keeper.StoreRandomContract(t, ctx, keepers, &mock)
	newInfo4 := wasmKeeper.GetCodeInfo(ctx, contract4.CodeID)
	legacyInfo4 := v3.CodeInfo{
		CodeHash:          newInfo4.CodeHash,
		Creator:           newInfo4.Creator,
		InstantiateConfig: allowNobodyPermission,
	}

	store.Set(types.GetCodeKey(contract4.CodeID), cdc.MustMarshal(&legacyInfo4))

	// set only address permission params
	ps := v3.Params{
		CodeUploadAccess: v3.AccessConfig{
			Permission: v3.AccessTypeOnlyAddress,
			Address:    creator.String(),
		},
		InstantiateDefaultPermission: v3.AccessTypeOnlyAddress,
	}

	bz, err := cdc.Marshal(&ps)
	require.NoError(t, err)
	store.Set(types.ParamsKey, bz)

	// when
	err = keeper.NewMigrator(*wasmKeeper, nil).Migrate3to4(ctx)

	// then
	require.NoError(t, err)

	expParams := types.Params{
		CodeUploadAccess:             types.AccessTypeAnyOfAddresses.With(creator),
		InstantiateDefaultPermission: types.AccessTypeAnyOfAddresses,
	}

	// params are migrated
	assert.Equal(t, expParams, wasmKeeper.GetParams(ctx))

	// access config for only address is migrated
	info1 := wasmKeeper.GetCodeInfo(ctx, contract1.CodeID)
	assert.Equal(t, types.AccessTypeAnyOfAddresses.With(creator), info1.InstantiateConfig)

	// access config for any addresses is not migrated
	info2 := wasmKeeper.GetCodeInfo(ctx, contract2.CodeID)
	assert.Equal(t, types.AccessTypeAnyOfAddresses.With(creator), info2.InstantiateConfig)

	// access config for allow everybody is not migrated
	info3 := wasmKeeper.GetCodeInfo(ctx, contract3.CodeID)
	assert.Equal(t, types.AllowEverybody, info3.InstantiateConfig)

	// access config for allow nobody is not migrated
	info4 := wasmKeeper.GetCodeInfo(ctx, contract4.CodeID)
	assert.Equal(t, types.AllowNobody, info4.InstantiateConfig)
}
