// This file only used to generate mocks

package testutil

import (
	"context"

	addresscodec "cosmossdk.io/core/address"
	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// AccountKeeper extends gov's actual expected AccountKeeper with additional
// methods used in tests.
type AccountKeeper interface {
	AddressCodec() addresscodec.Codec

	GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI

	GetModuleAddress(name string) sdk.AccAddress
	GetModuleAccount(ctx context.Context, name string) sdk.ModuleAccountI

	// TODO remove with genesis 2-phases refactor https://github.com/cosmos/cosmos-sdk/issues/2862
	SetModuleAccount(context.Context, sdk.ModuleAccountI)

	IterateAccounts(ctx context.Context, cb func(account sdk.AccountI) (stop bool))
}

// BankKeeper extends gov's actual expected BankKeeper with additional
// methods used in tests.
type BankKeeper interface {
	GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	GetBalance(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin
	MintCoins(ctx context.Context, moduleName string, amt sdk.Coins) error
	BurnCoins(ctx context.Context, address []byte, amt sdk.Coins) error
	LockedCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	SendCoinsFromModuleToAccount(ctx context.Context, senderModule string, recipientAddr sdk.AccAddress, amt sdk.Coins) error
	SendCoinsFromModuleToModule(ctx context.Context, senderModule, recipientModule string, amt sdk.Coins) error
	SendCoinsFromAccountToModule(ctx context.Context, senderAddr sdk.AccAddress, recipientModule string, amt sdk.Coins) error
}

// PoolKeeper extends the gov's actual expected PoolKeeper.
type PoolKeeper interface {
	FundCommunityPool(ctx context.Context, amount sdk.Coins, sender []byte) error
}

// StakingKeeper extends gov's actual expected StakingKeeper with additional
// methods used in tests.
type StakingKeeper interface {
	ValidatorAddressCodec() addresscodec.Codec
	// iterate through bonded validators by operator address, execute func for each validator
	IterateBondedValidatorsByPower(
		context.Context, func(index int64, validator sdk.ValidatorI) (stop bool),
	) error

	TotalBondedTokens(context.Context) (math.Int, error) // total bonded tokens within the validator set
	IterateDelegations(
		ctx context.Context, delegator sdk.AccAddress,
		fn func(index int64, delegation sdk.DelegationI) (stop bool),
	) error

	BondDenom(ctx context.Context) (string, error)
	TokensFromConsensusPower(ctx context.Context, power int64) math.Int
}
