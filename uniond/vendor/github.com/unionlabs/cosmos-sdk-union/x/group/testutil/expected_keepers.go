// This file only used to generate mocks

package testutil

import (
	"context"

	address "cosmossdk.io/core/address"
	bank "cosmossdk.io/x/bank/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// AccountKeeper extends `AccountKeeper` from expected_keepers.
type AccountKeeper interface {
	AddressCodec() address.Codec

	// NewAccount returns a new account with the next account number. Does not save the new account to the store.
	NewAccount(context.Context, sdk.AccountI) sdk.AccountI

	// GetAccount retrieves an account from the store.
	GetAccount(context.Context, sdk.AccAddress) sdk.AccountI

	// SetAccount sets an account in the store.
	SetAccount(context.Context, sdk.AccountI)

	// RemoveAccount Remove an account in the store.
	RemoveAccount(ctx context.Context, acc sdk.AccountI)
}

// BankKeeper extends bank `MsgServer` to mock `Send` and to register handlers in MsgServiceRouter
type BankKeeper interface {
	SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	bank.MsgServer

	MintCoins(ctx context.Context, moduleName string, amt sdk.Coins) error
	SendCoinsFromModuleToAccount(ctx context.Context, senderModule string, recipientAddr sdk.AccAddress, amt sdk.Coins) error
	GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins
}
