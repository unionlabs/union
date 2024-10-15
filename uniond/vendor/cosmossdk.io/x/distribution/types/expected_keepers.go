package types

import (
	"context"

	"cosmossdk.io/core/address"
	stakingtypes "cosmossdk.io/x/staking/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// AccountKeeper defines the expected account keeper used for simulations (noalias)
type AccountKeeper interface {
	AddressCodec() address.Codec
	GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
	GetModuleAddress(name string) sdk.AccAddress
	GetModuleAccount(ctx context.Context, name string) sdk.ModuleAccountI
	// TODO remove with genesis 2-phases refactor https://github.com/cosmos/cosmos-sdk/issues/2862
	SetModuleAccount(context.Context, sdk.ModuleAccountI)
}

// BankKeeper defines the expected interface needed to retrieve account balances.
type BankKeeper interface {
	MintCoins(ctx context.Context, moduleName string, amt sdk.Coins) error
	GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins

	SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins

	SendCoinsFromModuleToModule(ctx context.Context, senderModule, recipientModule string, amt sdk.Coins) error
	SendCoinsFromModuleToAccount(ctx context.Context, senderModule string, recipientAddr sdk.AccAddress, amt sdk.Coins) error
	SendCoinsFromAccountToModule(ctx context.Context, senderAddr sdk.AccAddress, recipientModule string, amt sdk.Coins) error

	BlockedAddr(addr sdk.AccAddress) bool
	IsSendEnabledDenom(ctx context.Context, denom string) bool
}

// StakingKeeper expected staking keeper (noalias)
type StakingKeeper interface {
	ValidatorAddressCodec() address.Codec
	ConsensusAddressCodec() address.Codec
	BondDenom(ctx context.Context) (string, error)

	// iterate through validators by operator address, execute func for each validator
	IterateValidators(context.Context,
		func(index int64, validator sdk.ValidatorI) (stop bool)) error

	Validator(context.Context, sdk.ValAddress) (sdk.ValidatorI, error)            // get a particular validator by operator address
	ValidatorByConsAddr(context.Context, sdk.ConsAddress) (sdk.ValidatorI, error) // get a particular validator by consensus address

	// Delegation allows for getting a particular delegation for a given validator
	// and delegator outside the scope of the staking module.
	Delegation(context.Context, sdk.AccAddress, sdk.ValAddress) (sdk.DelegationI, error)

	IterateDelegations(ctx context.Context, delegator sdk.AccAddress,
		fn func(index int64, delegation sdk.DelegationI) (stop bool)) error

	GetAllSDKDelegations(ctx context.Context) ([]stakingtypes.Delegation, error)
	GetAllValidators(ctx context.Context) ([]stakingtypes.Validator, error)
	GetAllDelegatorDelegations(ctx context.Context, delegator sdk.AccAddress) ([]stakingtypes.Delegation, error)
}

// StakingHooks event hooks for staking validator object (noalias)
type StakingHooks interface {
	AfterValidatorCreated(ctx context.Context, valAddr sdk.ValAddress) error // Must be called when a validator is created
	AfterDelegationModified(ctx context.Context, delAddr sdk.AccAddress, valAddr sdk.ValAddress) error
}
