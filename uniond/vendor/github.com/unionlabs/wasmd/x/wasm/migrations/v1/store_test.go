package v1_test

import (
	"bytes"
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"

	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestMigrate1To2(t *testing.T) {
	AvailableCapabilities := []string{"iterator", "staking", "stargate", "cosmwasm_1_1"}
	ctx, keepers := keeper.CreateTestInput(t, false, AvailableCapabilities)
	wasmKeeper := keepers.WasmKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := sdk.AccAddress(bytes.Repeat([]byte{1}, address.Len))
	keepers.Faucet.Fund(ctx, creator, deposit...)
	example := keeper.StoreHackatomExampleContract(t, ctx, keepers)

	initMsg := keeper.HackatomExampleInitMsg{
		Verifier:    keeper.RandomAccountAddress(t),
		Beneficiary: keeper.RandomAccountAddress(t),
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	em := sdk.NewEventManager()

	// create with no balance is also legal
	gotContractAddr1, _, err := keepers.ContractKeeper.Instantiate(ctx.WithEventManager(em), example.CodeID, creator, nil, initMsgBz, "demo contract 1", nil)
	require.NoError(t, err)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// create with no balance is also legal
	gotContractAddr2, _, err := keepers.ContractKeeper.Instantiate(ctx.WithEventManager(em), example.CodeID, creator, nil, initMsgBz, "demo contract 1", nil)
	require.NoError(t, err)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)

	// create with no balance is also legal
	gotContractAddr3, _, err := keepers.ContractKeeper.Instantiate(ctx.WithEventManager(em), example.CodeID, creator, nil, initMsgBz, "demo contract 1", nil)
	require.NoError(t, err)

	info1 := wasmKeeper.GetContractInfo(ctx, gotContractAddr1)
	info2 := wasmKeeper.GetContractInfo(ctx, gotContractAddr2)
	info3 := wasmKeeper.GetContractInfo(ctx, gotContractAddr3)

	// remove key
	ctx.KVStore(keepers.WasmStoreKey).Delete(types.GetContractByCreatorSecondaryIndexKey(creator, info1.Created.Bytes(), gotContractAddr1))
	ctx.KVStore(keepers.WasmStoreKey).Delete(types.GetContractByCreatorSecondaryIndexKey(creator, info2.Created.Bytes(), gotContractAddr2))
	ctx.KVStore(keepers.WasmStoreKey).Delete(types.GetContractByCreatorSecondaryIndexKey(creator, info3.Created.Bytes(), gotContractAddr3))

	// migrator
	err = keeper.NewMigrator(*wasmKeeper, nil).Migrate1to2(ctx)
	require.NoError(t, err)

	// check new store
	var allContract []string
	wasmKeeper.IterateContractsByCreator(ctx, creator, func(addr sdk.AccAddress) bool {
		allContract = append(allContract, addr.String())
		return false
	})

	require.Equal(t, []string{gotContractAddr1.String(), gotContractAddr2.String(), gotContractAddr3.String()}, allContract)
}
