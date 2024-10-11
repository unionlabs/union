package v7

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// MigrateLocalhostClient initialises the 09-localhost client state and sets it in state.
func MigrateLocalhostClient(ctx sdk.Context, clientKeeper ClientKeeper) error {
	return clientKeeper.CreateLocalhostClient(ctx)
}
