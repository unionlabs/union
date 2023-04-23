package v2

import (
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	v1 "github.com/cosmos/cosmos-sdk/x/distribution/migrations/v1"
)

// MigrateStore performs in-place store migrations from v0.40 to v0.43. The
// migration includes:
//
// - Change addresses to be length-prefixed.
func MigrateStore(ctx sdk.Context, storeKey storetypes.StoreKey) error {
	store := ctx.KVStore(storeKey)
	MigratePrefixAddress(store, v1.ValidatorOutstandingRewardsPrefix)
	MigratePrefixAddress(store, v1.DelegatorWithdrawAddrPrefix)
	MigratePrefixAddressAddress(store, v1.DelegatorStartingInfoPrefix)
	MigratePrefixAddressBytes(store, v1.ValidatorHistoricalRewardsPrefix)
	MigratePrefixAddress(store, v1.ValidatorCurrentRewardsPrefix)
	MigratePrefixAddress(store, v1.ValidatorAccumulatedCommissionPrefix)
	MigratePrefixAddressBytes(store, v1.ValidatorSlashEventPrefix)

	return nil
}
