package v7

import (
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// ClientKeeper expected IBC client keeper
type ClientKeeper interface {
	GetClientState(ctx sdk.Context, clientID string) (exported.ClientState, bool)
	SetClientState(ctx sdk.Context, clientID string, clientState exported.ClientState)
	ClientStore(ctx sdk.Context, clientID string) storetypes.KVStore
	CreateLocalhostClient(ctx sdk.Context) error
}
