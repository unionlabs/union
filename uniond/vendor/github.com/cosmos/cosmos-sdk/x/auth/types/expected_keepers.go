package types

import (
	"context"

	"cosmossdk.io/core/transaction"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// BankKeeper defines the contract needed for supply related APIs (noalias)
type BankKeeper interface {
	IsSendEnabledCoins(ctx context.Context, coins ...sdk.Coin) error
	SendCoins(ctx context.Context, from, to sdk.AccAddress, amt sdk.Coins) error
	SendCoinsFromAccountToModule(ctx context.Context, senderAddr sdk.AccAddress, recipientModule string, amt sdk.Coins) error
}

// AccountsModKeeper defines the contract for x/accounts APIs
type AccountsModKeeper interface {
	SendModuleMessage(ctx context.Context, sender []byte, msg transaction.Msg) (transaction.Msg, error)
	IsAccountsModuleAccount(ctx context.Context, accountAddr []byte) bool
	NextAccountNumber(ctx context.Context) (accNum uint64, err error)

	// Query is used to query an account
	Query(
		ctx context.Context,
		accountAddr []byte,
		queryRequest transaction.Msg,
	) (transaction.Msg, error)

	// InitAccountNumberSeqUnsafe is use to set accounts module account number with value
	// of auth module current account number
	InitAccountNumberSeqUnsafe(ctx context.Context, currentAccNum uint64) error

	// MigrateLegacyAccount migrates the given account to an x/accounts' account.
	MigrateLegacyAccount(
		ctx context.Context,
		addr []byte, // The current address of the account
		accNum uint64, // The current account number
		accType string, // The account type to migrate to
		msg transaction.Msg, // The init msg of the account type we're migrating to
	) (transaction.Msg, error)
}
