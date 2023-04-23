package v2

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

// pruneZeroBalancesJSON removes the zero balance addresses from exported genesis.
func pruneZeroBalancesJSON(oldBalances []types.Balance) []types.Balance {
	var balances []types.Balance

	for _, b := range oldBalances {
		if !b.Coins.IsZero() {
			b.Coins = sdk.NewCoins(b.Coins...) // prunes zero denom.
			balances = append(balances, b)
		}
	}

	return balances
}

// MigrateJSON accepts exported v0.40 x/bank genesis state and migrates it to
// v0.43 x/bank genesis state. The migration includes:
// - Prune balances & supply with zero coins (ref: https://github.com/cosmos/cosmos-sdk/pull/9229)
func MigrateJSON(oldState *types.GenesisState) *types.GenesisState {
	return &types.GenesisState{
		Params:        oldState.Params,
		Balances:      pruneZeroBalancesJSON(oldState.Balances),
		Supply:        sdk.NewCoins(oldState.Supply...), // NewCoins used here to remove zero coin denoms from supply.
		DenomMetadata: oldState.DenomMetadata,
	}
}
