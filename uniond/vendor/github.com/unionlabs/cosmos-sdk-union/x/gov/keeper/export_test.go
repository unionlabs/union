package keeper

import (
	v1 "cosmossdk.io/x/gov/types/v1"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// ValidateInitialDeposit is a helper function used only in deposit tests which returns the same
// functionality of validateInitialDeposit private function.
func (k Keeper) ValidateInitialDeposit(ctx sdk.Context, initialDeposit sdk.Coins, proposalType v1.ProposalType) error {
	params, err := k.Params.Get(ctx)
	if err != nil {
		return err
	}

	return k.validateInitialDeposit(params, initialDeposit, proposalType)
}
