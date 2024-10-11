package feegrant

import (
	"context"

	"cosmossdk.io/core/address"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// AccountKeeper defines the expected auth Account Keeper (noalias)
type AccountKeeper interface {
	AddressCodec() address.Codec

	GetModuleAddress(moduleName string) sdk.AccAddress
	GetModuleAccount(ctx context.Context, moduleName string) sdk.ModuleAccountI

	NewAccountWithAddress(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	SetAccount(ctx context.Context, acc sdk.AccountI)
}

// BankKeeper defines the expected supply Keeper (noalias)
type BankKeeper interface {
	SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	SendCoinsFromAccountToModule(ctx context.Context, senderAddr sdk.AccAddress, recipientModule string, amt sdk.Coins) error
}
