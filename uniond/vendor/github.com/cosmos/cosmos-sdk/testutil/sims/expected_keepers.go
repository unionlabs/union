package sims

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type BankKeeper interface {
	SendCoinsFromModuleToAccount(ctx context.Context, senderModule string, recipientAddr sdk.AccAddress, amt sdk.Coins) error
	MintCoins(ctx context.Context, moduleName string, amt sdk.Coins) error
}

// StakingKeeper is a subset of the staking keeper's public interface that
// provides the staking bond denom. It is used in arguments in this package's
// functions so that a mock staking keeper can be passed instead of the real one.
type StakingKeeper interface {
	BondDenom(ctx context.Context) (string, error)
}
