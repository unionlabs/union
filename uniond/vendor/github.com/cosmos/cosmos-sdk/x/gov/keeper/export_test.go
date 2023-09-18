package keeper

import sdk "github.com/cosmos/cosmos-sdk/types"

// ValidateInitialDeposit is a helper function used only in deposit tests which returns the same
// functionality of validateInitialDeposit private function.
func (k Keeper) ValidateInitialDeposit(ctx sdk.Context, initialDeposit sdk.Coins) error {
	return k.validateInitialDeposit(ctx, initialDeposit)
}
