package types

import (
	"context"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// ClientKeeper defines the expected client keeper
type ClientKeeper interface {
	ClientStore(ctx context.Context, clientID string) storetypes.KVStore
	GetClientState(ctx context.Context, clientID string) (exported.ClientState, bool)
	SetClientState(ctx context.Context, clientID string, clientState exported.ClientState)
}
