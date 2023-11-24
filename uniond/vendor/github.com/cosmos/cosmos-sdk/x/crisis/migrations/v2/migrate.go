package v2

import (
	storetypes "cosmossdk.io/core/store"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/crisis/exported"
)

const (
	ModuleName = "crisis"
)

var (
	ConstantFee    = []byte("ConstantFee")
	ConstantFeeKey = []byte{0x01}
)

// MigrateStore migrates the x/crisis module state from the consensus version 1 to
// version 2. Specifically, it takes the `ConstantFee` parameter that is currently stored
// and managed by the x/params module and stores it directly into the x/crisis
// module state.
func MigrateStore(ctx sdk.Context, storeService storetypes.KVStoreService, legacySubspace exported.Subspace, cdc codec.BinaryCodec) error {
	store := storeService.OpenKVStore(ctx)
	var currConstantFee sdk.Coin
	legacySubspace.Get(ctx, ConstantFee, &currConstantFee)

	if !currConstantFee.IsValid() {
		return errors.ErrInvalidCoins.Wrap("constant fee")
	}

	bz, err := cdc.Marshal(&currConstantFee)
	if err != nil {
		return err
	}

	return store.Set(ConstantFeeKey, bz)
}
