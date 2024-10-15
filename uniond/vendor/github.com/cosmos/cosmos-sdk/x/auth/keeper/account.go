package keeper

import (
	"context"
	"errors"

	"cosmossdk.io/collections"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NewAccountWithAddress implements AccountKeeperI.
func (ak AccountKeeper) NewAccountWithAddress(ctx context.Context, addr sdk.AccAddress) sdk.AccountI {
	acc := ak.proto()
	err := acc.SetAddress(addr)
	if err != nil {
		panic(err)
	}

	return ak.NewAccount(ctx, acc)
}

// NewAccount sets the next account number to a given account interface
func (ak AccountKeeper) NewAccount(ctx context.Context, acc sdk.AccountI) sdk.AccountI {
	accNum, err := ak.AccountsModKeeper.NextAccountNumber(ctx)
	if err != nil {
		panic(err)
	}

	if err := acc.SetAccountNumber(accNum); err != nil {
		panic(err)
	}

	return acc
}

// HasAccount implements AccountKeeperI.
func (ak AccountKeeper) HasAccount(ctx context.Context, addr sdk.AccAddress) bool {
	has, _ := ak.Accounts.Has(ctx, addr)
	return has || ak.AccountsModKeeper.IsAccountsModuleAccount(ctx, addr)
}

// GetAccount implements AccountKeeperI.
func (ak AccountKeeper) GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI {
	acc, err := ak.Accounts.Get(ctx, addr)
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		panic(err)
	}
	return acc
}

// SetAccount implements AccountKeeperI.
func (ak AccountKeeper) SetAccount(ctx context.Context, acc sdk.AccountI) {
	err := ak.Accounts.Set(ctx, acc.GetAddress(), acc)
	if err != nil {
		panic(err)
	}
}

// RemoveAccount removes an account for the account mapper store.
// NOTE: this will cause supply invariant violation if called
func (ak AccountKeeper) RemoveAccount(ctx context.Context, acc sdk.AccountI) {
	err := ak.Accounts.Remove(ctx, acc.GetAddress())
	if err != nil {
		panic(err)
	}
}
