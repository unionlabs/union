package v7

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// MigrateLocalhostConnection creates the sentinel localhost connection end to enable
// localhost ibc functionality.
func MigrateLocalhostConnection(ctx sdk.Context, connectionKeeper ConnectionKeeper) {
	connectionKeeper.CreateSentinelLocalhostConnection(ctx)
}
