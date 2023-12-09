package v2

import (
	corestoretypes "cosmossdk.io/core/store"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/exported"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// MigrateStore migrates the x/wasm module state from the consensus version 2 to
// version 3. Specifically, it takes the parameters that are currently stored
// and managed by the x/params module and stores them directly into the x/wasm
// module state.
func MigrateStore(ctx sdk.Context, storeService corestoretypes.KVStoreService, legacySubspace exported.Subspace, cdc codec.BinaryCodec) error {
	store := storeService.OpenKVStore(ctx)
	var currParams Params
	legacySubspace.GetParamSet(ctx, &currParams)
	bz, err := cdc.Marshal(&currParams)
	if err != nil {
		return err
	}

	return store.Set(types.ParamsKey, bz)
}
