package types

import (
	"context"

	storetypes "cosmossdk.io/store/types"
	paramtypes "cosmossdk.io/x/params/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// ClientKeeper expected account IBC client keeper
type ClientKeeper interface {
	GetClientStatus(ctx context.Context, clientState exported.ClientState, clientID string) exported.Status
	GetClientState(ctx context.Context, clientID string) (exported.ClientState, bool)
	GetClientConsensusState(ctx context.Context, clientID string, height exported.Height) (exported.ConsensusState, bool)
	GetSelfConsensusState(ctx context.Context, height exported.Height) (exported.ConsensusState, error)
	ValidateSelfClient(ctx context.Context, clientState exported.ClientState) error
	IterateClientStates(ctx context.Context, prefix []byte, cb func(string, exported.ClientState) bool)
	ClientStore(ctx context.Context, clientID string) storetypes.KVStore
}

// ParamSubspace defines the expected Subspace interface for module parameters.
type ParamSubspace interface {
	GetParamSet(ctx sdk.Context, ps paramtypes.ParamSet)
}
