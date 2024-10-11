package v4

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/x/distribution/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// MigrateFunds migrates the distribution module funds to pool module
func MigrateFunds(ctx context.Context, bankKeeper types.BankKeeper, feePool types.FeePool, macc, poolMacc sdk.ModuleAccountI) (types.FeePool, error) {
	poolBal, remainder := feePool.CommunityPool.TruncateDecimal()

	distrbalances := bankKeeper.GetAllBalances(ctx, macc.GetAddress())
	if distrbalances.IsZero() || distrbalances.IsAllLT(poolBal) {
		return types.FeePool{}, fmt.Errorf("%s module account balance is less than FeePool balance", macc.GetName())
	}

	// transfer feepool funds from the distribution module account to pool module account
	if err := bankKeeper.SendCoinsFromModuleToModule(ctx, macc.GetName(), poolMacc.GetName(), poolBal); err != nil {
		return types.FeePool{}, err
	}

	// check the migrated balance from pool module account is same as fee pool balance
	balances := bankKeeper.GetAllBalances(ctx, poolMacc.GetAddress())
	if !balances.Equal(poolBal) {
		return types.FeePool{}, errors.New("pool module account balance is not same as FeePool balance after migration")
	}

	return types.FeePool{DecimalPool: remainder}, nil
}
