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
	if err := acc.SetAccountNumber(ak.NextAccountNumber(ctx)); err != nil {
		panic(err)
	}

	return acc
}

// HasAccount implements AccountKeeperI.
func (ak AccountKeeper) HasAccount(ctx context.Context, addr sdk.AccAddress) bool {
	has, _ := ak.Accounts.Has(ctx, addr)
	return has
}

// GetAccount implements AccountKeeperI.
func (ak AccountKeeper) GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI {
	acc, err := ak.Accounts.Get(ctx, addr)
	if err != nil && !errors.Is(err, collections.ErrNotFound) {
		panic(err)
	}
	return acc
}

// GetAllAccounts returns all accounts in the accountKeeper.
func (ak AccountKeeper) GetAllAccounts(ctx context.Context) (accounts []sdk.AccountI) {
	ak.IterateAccounts(ctx, func(acc sdk.AccountI) (stop bool) {
		accounts = append(accounts, acc)
		return false
	})

	return accounts
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

// IterateAccounts iterates over all the stored accounts and performs a callback function.
// Stops iteration when callback returns true.
func (ak AccountKeeper) IterateAccounts(ctx context.Context, cb func(account sdk.AccountI) (stop bool)) {
	err := ak.Accounts.Walk(ctx, nil, func(_ sdk.AccAddress, value sdk.AccountI) (bool, error) {
		return cb(value), nil
	})
	if err != nil {
		panic(err)
	}
}
