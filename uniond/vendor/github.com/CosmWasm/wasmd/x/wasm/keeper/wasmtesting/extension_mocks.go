package wasmtesting

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type MockCoinTransferrer struct {
	TransferCoinsFn func(ctx sdk.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) error
}

func (m *MockCoinTransferrer) TransferCoins(ctx sdk.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) error {
	if m.TransferCoinsFn == nil {
		panic("not expected to be called")
	}
	return m.TransferCoinsFn(ctx, fromAddr, toAddr, amt)
}

type AccountPrunerMock struct {
	CleanupExistingAccountFn func(ctx sdk.Context, existingAccount sdk.AccountI) (handled bool, err error)
}

func (m AccountPrunerMock) CleanupExistingAccount(ctx sdk.Context, existingAccount sdk.AccountI) (handled bool, err error) {
	if m.CleanupExistingAccountFn == nil {
		panic("not expected to be called")
	}
	return m.CleanupExistingAccountFn(ctx, existingAccount)
}
