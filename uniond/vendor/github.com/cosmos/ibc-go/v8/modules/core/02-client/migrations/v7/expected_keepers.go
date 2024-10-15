package v7

import (
	"context"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// ClientKeeper expected IBC client keeper
type ClientKeeper interface {
	GetClientState(ctx context.Context, clientID string) (exported.ClientState, bool)
	SetClientState(ctx context.Context, clientID string, clientState exported.ClientState)
	ClientStore(ctx context.Context, clientID string) storetypes.KVStore
	CreateLocalhostClient(ctx context.Context) error
}
