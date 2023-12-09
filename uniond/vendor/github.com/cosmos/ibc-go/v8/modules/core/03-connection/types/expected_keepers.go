package types

import (
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// ClientKeeper expected account IBC client keeper
type ClientKeeper interface {
	GetClientStatus(ctx sdk.Context, clientState exported.ClientState, clientID string) exported.Status
	GetClientState(ctx sdk.Context, clientID string) (exported.ClientState, bool)
	GetClientConsensusState(ctx sdk.Context, clientID string, height exported.Height) (exported.ConsensusState, bool)
	GetSelfConsensusState(ctx sdk.Context, height exported.Height) (exported.ConsensusState, error)
	ValidateSelfClient(ctx sdk.Context, clientState exported.ClientState) error
	IterateClientStates(ctx sdk.Context, prefix []byte, cb func(string, exported.ClientState) bool)
	ClientStore(ctx sdk.Context, clientID string) storetypes.KVStore
}

// ParamSubspace defines the expected Subspace interface for module parameters.
type ParamSubspace interface {
	GetParamSet(ctx sdk.Context, ps paramtypes.ParamSet)
}
