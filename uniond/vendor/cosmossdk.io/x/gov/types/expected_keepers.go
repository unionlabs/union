package types

import (
	"context"

	addresscodec "cosmossdk.io/core/address"
	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// StakingKeeper expected staking keeper (Validator and Delegator sets) (noalias)
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
}

// AccountKeeper defines the expected account keeper (noalias)
type AccountKeeper interface {
	AddressCodec() addresscodec.Codec

	GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI

	GetModuleAddress(name string) sdk.AccAddress
	GetModuleAccount(ctx context.Context, name string) sdk.ModuleAccountI

	// TODO remove with genesis 2-phases refactor https://github.com/cosmos/cosmos-sdk/issues/2862
	SetModuleAccount(context.Context, sdk.ModuleAccountI)
}

// BankKeeper defines the expected interface needed to retrieve account balances.
type BankKeeper interface {
	GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	GetBalance(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin
	LockedCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins

	SendCoinsFromModuleToAccount(ctx context.Context, senderModule string, recipientAddr sdk.AccAddress, amt sdk.Coins) error
	SendCoinsFromAccountToModule(ctx context.Context, senderAddr sdk.AccAddress, recipientModule string, amt sdk.Coins) error
	BurnCoins(context.Context, []byte, sdk.Coins) error
}

// PoolKeeper defines the expected interface needed to fund & distribute pool balances.
type PoolKeeper interface {
	FundCommunityPool(ctx context.Context, amount sdk.Coins, sender []byte) error
}

// Event Hooks
// These can be utilized to communicate between a governance keeper and another
// keepers.

// GovHooks event hooks for governance proposal object (noalias)
type GovHooks interface {
	AfterProposalSubmission(ctx context.Context, proposalID uint64) error                            // Must be called after proposal is submitted
	AfterProposalDeposit(ctx context.Context, proposalID uint64, depositorAddr sdk.AccAddress) error // Must be called after a deposit is made
	AfterProposalVote(ctx context.Context, proposalID uint64, voterAddr sdk.AccAddress) error        // Must be called after a vote on a proposal is cast
	AfterProposalFailedMinDeposit(ctx context.Context, proposalID uint64) error                      // Must be called when proposal fails to reach min deposit
	AfterProposalVotingPeriodEnded(ctx context.Context, proposalID uint64) error                     // Must be called when proposal's finishes it's voting period
}

type GovHooksWrapper struct{ GovHooks }

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (GovHooksWrapper) IsOnePerModuleType() {}
