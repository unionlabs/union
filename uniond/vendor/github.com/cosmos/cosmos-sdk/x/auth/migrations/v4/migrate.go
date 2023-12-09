package v4

import (
	storetypes "cosmossdk.io/core/store"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/exported"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

var ParamsKey = []byte{0x00}

// Migrate migrates the x/auth module state from the consensus version 3 to
// version 4. Specifically, it takes the parameters that are currently stored
// and managed by the x/params modules and stores them directly into the x/auth
// module state.
func Migrate(ctx sdk.Context, storeService storetypes.KVStoreService, legacySubspace exported.Subspace, cdc codec.BinaryCodec) error {
	store := storeService.OpenKVStore(ctx)
	var currParams types.Params
	legacySubspace.GetParamSet(ctx, &currParams)

	if err := currParams.Validate(); err != nil {
		return err
	}

	bz := cdc.MustMarshal(&currParams)
	store.Set(ParamsKey, bz)

	return nil
}
