package migrations

import (
	"context"

	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// ClientKeeper expected account IBC client keeper
type ClientKeeper interface {
	GetClientState(ctx context.Context, clientID string) (exported.ClientState, bool)
	IterateClientStates(ctx context.Context, prefix []byte, cb func(string, exported.ClientState) bool)
	ClientStore(ctx context.Context, clientID string) storetypes.KVStore
	Logger(ctx context.Context) log.Logger
}
