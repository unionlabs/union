package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	clientkeeper "github.com/cosmos/ibc-go/v7/modules/core/02-client/keeper"
	connectiontypes "github.com/cosmos/ibc-go/v7/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

type Keeper struct {
	clientKeeper connectiontypes.ClientKeeper
}

func NewKeeper(clientKeeper clientkeeper.Keeper) connectiontypes.ClientKeeper {
	return Keeper{
		clientKeeper: clientKeeper,
	}
}

func (k Keeper) GetClientState(ctx sdk.Context, clientID string) (exported.ClientState, bool) {
	return k.clientKeeper.GetClientState(ctx, clientID)
}

func (k Keeper) GetClientConsensusState(ctx sdk.Context, clientID string, height exported.Height) (exported.ConsensusState, bool) {
	return k.clientKeeper.GetClientConsensusState(ctx, clientID, height)
}

func (k Keeper) GetSelfConsensusState(ctx sdk.Context, height exported.Height, clientType string) (exported.ConsensusState, error) {
	return k.clientKeeper.GetSelfConsensusState(ctx, height, clientType)
}

func (k Keeper) ValidateSelfClient(ctx sdk.Context, clientState exported.ClientState) error {
	// we don't have to verify cometbls client state
	return nil
}

func (k Keeper) IterateClientStates(ctx sdk.Context, prefix []byte, cb func(clientID string, cs exported.ClientState) bool) {
	k.clientKeeper.IterateClientStates(ctx, prefix, cb)
}

func (k Keeper) ClientStore(ctx sdk.Context, clientID string) sdk.KVStore {
	return k.clientKeeper.ClientStore(ctx, clientID)
}

func (k Keeper) GetClientStatus(ctx sdk.Context, clientState exported.ClientState, clientID string) exported.Status {
	return k.clientKeeper.GetClientStatus(ctx, clientState, clientID)
}
