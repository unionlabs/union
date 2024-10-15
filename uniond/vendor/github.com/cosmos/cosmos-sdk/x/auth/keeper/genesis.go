package keeper

import (
	"context"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

// InitGenesis - Init store state from genesis data
//
// CONTRACT: old coins from the FeeCollectionKeeper need to be transferred through
// a genesis port script to the new fee collector account
func (ak AccountKeeper) InitGenesis(ctx context.Context, data types.GenesisState) error {
	if err := ak.Params.Set(ctx, data.Params); err != nil {
		return err
	}

	accounts, err := types.UnpackAccounts(data.Accounts)
	if err != nil {
		return err
	}
	accounts = types.SanitizeGenesisAccounts(accounts)

	// Set the accounts and make sure the global account number matches the largest account number (even if zero).
	var lastAccNum *uint64
	for _, acc := range accounts {
		accNum := acc.GetAccountNumber()
		for lastAccNum == nil || *lastAccNum < accNum {
			n, err := ak.AccountsModKeeper.NextAccountNumber(ctx)
			if err != nil {
				return err
			}
			lastAccNum = &n
		}
		ak.SetAccount(ctx, acc)
	}

	ak.GetModuleAccount(ctx, types.FeeCollectorName)
	return nil
}

// ExportGenesis returns a GenesisState for a given context and keeper
func (ak AccountKeeper) ExportGenesis(ctx context.Context) (*types.GenesisState, error) {
	params := ak.GetParams(ctx)

	var genAccounts types.GenesisAccounts
	err := ak.Accounts.Walk(ctx, nil, func(key sdk.AccAddress, value sdk.AccountI) (stop bool, err error) {
		genAcc, ok := value.(types.GenesisAccount)
		if !ok {
			return true, fmt.Errorf("unable to convert account with address %s into a genesis account: type %T", key, value)
		}
		genAccounts = append(genAccounts, genAcc)
		return false, nil
	})
	return types.NewGenesisState(params, genAccounts), err
}
