package keeper

import (
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// RegisterInvariants registers all governance invariants
func RegisterInvariants(ir sdk.InvariantRegistry, keeper *Keeper, bk types.BankKeeper) {
	ir.RegisterRoute(types.ModuleName, "module-account", ModuleAccountInvariant(keeper, bk))
}

// ModuleAccountInvariant checks that the module account coins reflects the sum of
// deposit amounts held on store.
func ModuleAccountInvariant(keeper *Keeper, bk types.BankKeeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		var expectedDeposits sdk.Coins

		err := keeper.Deposits.Walk(ctx, nil, func(key collections.Pair[uint64, sdk.AccAddress], value v1.Deposit) (stop bool, err error) {
			expectedDeposits = expectedDeposits.Add(value.Amount...)
			return false, nil
		})
		if err != nil {
			panic(err)
		}

		macc := keeper.GetGovernanceAccount(ctx)
		balances := bk.GetAllBalances(ctx, macc.GetAddress())

		// Require that the deposit balances are <= than the x/gov module's total
		// balances. We use the <= operator since external funds can be sent to x/gov
		// module's account and so the balance can be larger.
		broken := !balances.IsAllGTE(expectedDeposits)

		return sdk.FormatInvariant(types.ModuleName, "deposits",
			fmt.Sprintf("\tgov ModuleAccount coins: %s\n\tsum of deposit amounts:  %s\n",
				balances, expectedDeposits)), broken
	}
}
