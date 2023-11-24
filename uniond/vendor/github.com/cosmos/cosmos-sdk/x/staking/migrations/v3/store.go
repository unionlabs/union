package v3

import (
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"
	"github.com/cosmos/cosmos-sdk/x/staking/exported"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

// subspace contains the method needed for migrations of the
// legacy Params subspace
type subspace interface {
	GetParamSet(ctx sdk.Context, ps paramtypes.ParamSet)
	HasKeyTable() bool
	WithKeyTable(paramtypes.KeyTable) paramtypes.Subspace
	Set(ctx sdk.Context, key []byte, value interface{})
}

// MigrateStore performs in-place store migrations from v0.43/v0.44/v0.45 to v0.46.
// The migration includes:
//
// - Setting the MinCommissionRate param in the paramstore
func MigrateStore(ctx sdk.Context, store storetypes.KVStore, cdc codec.BinaryCodec, paramstore exported.Subspace) error {
	migrateParamsStore(ctx, paramstore.(subspace))

	return nil
}

func migrateParamsStore(ctx sdk.Context, paramstore subspace) {
	if paramstore.HasKeyTable() {
		paramstore.Set(ctx, types.KeyMinCommissionRate, types.DefaultMinCommissionRate)
	} else {
		paramstore.WithKeyTable(types.ParamKeyTable())
		paramstore.Set(ctx, types.KeyMinCommissionRate, types.DefaultMinCommissionRate)
	}
}
