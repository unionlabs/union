package keeper

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

// RegisterInvariants registers all transfer invariants
func RegisterInvariants(ir sdk.InvariantRegistry, k *Keeper) {
	ir.RegisterRoute(types.ModuleName, "total-escrow-per-denom",
		TotalEscrowPerDenomInvariants(k))
}

// AllInvariants runs all invariants of the transfer module.
func AllInvariants(k *Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		return TotalEscrowPerDenomInvariants(k)(ctx)
	}
}

// TotalEscrowPerDenomInvariants checks that the total amount escrowed for
// each denom is not smaller than the amount stored in the state entry.
func TotalEscrowPerDenomInvariants(k *Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		var actualTotalEscrowed sdk.Coins

		expectedTotalEscrowed := k.GetAllTotalEscrowed(ctx)

		portID := k.GetPort(ctx)
		transferChannels := k.channelKeeper.GetAllChannelsWithPortPrefix(ctx, portID)
		for _, channel := range transferChannels {
			escrowAddress := types.GetEscrowAddress(portID, channel.ChannelId)
			escrowBalances := k.bankKeeper.GetAllBalances(ctx, escrowAddress)

			actualTotalEscrowed = actualTotalEscrowed.Add(escrowBalances...)
		}

		// the actual escrowed amount must be greater than or equal to the expected amount for all denominations
		if !actualTotalEscrowed.IsAllGTE(expectedTotalEscrowed) {
			return sdk.FormatInvariant(
				types.ModuleName,
				"total escrow per denom invariance",
				fmt.Sprintf("found denom(s) with total escrow amount lower than expected:\nactual total escrowed: %s\nexpected total escrowed: %s", actualTotalEscrowed, expectedTotalEscrowed)), true
		}

		return "", false
	}
}
