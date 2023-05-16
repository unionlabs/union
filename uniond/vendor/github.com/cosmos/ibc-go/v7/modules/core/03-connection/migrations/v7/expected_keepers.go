package v7

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// ConnectionKeeper expected IBC connection keeper
type ConnectionKeeper interface {
	CreateSentinelLocalhostConnection(ctx sdk.Context)
}
