package keeper

import (
	errorsmod "cosmossdk.io/errors"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/errors"
	clientkeeper "github.com/cosmos/ibc-go/v7/modules/core/02-client/keeper"
	clienttypes "github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v7/modules/core/03-connection/types"
	commitmenttypes "github.com/cosmos/ibc-go/v7/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
	wasmtypes "github.com/cosmos/ibc-go/v7/modules/light-clients/08-wasm/types"
)

type Keeper struct {
	cdc           codec.BinaryCodec
	clientKeeper  connectiontypes.ClientKeeper
	stakingKeeper clienttypes.StakingKeeper
}

func NewKeeper(cdc codec.BinaryCodec, clientKeeper clientkeeper.Keeper, stakingKeeper clienttypes.StakingKeeper) connectiontypes.ClientKeeper {
	return Keeper{
		cdc:           cdc,
		clientKeeper:  clientKeeper,
		stakingKeeper: stakingKeeper,
	}
}

func (k Keeper) GetClientState(ctx sdk.Context, clientID string) (exported.ClientState, bool) {
	return k.clientKeeper.GetClientState(ctx, clientID)
}

func (k Keeper) GetClientConsensusState(ctx sdk.Context, clientID string, height exported.Height) (exported.ConsensusState, bool) {
	return k.clientKeeper.GetClientConsensusState(ctx, clientID, height)
}

func (k Keeper) GetSelfConsensusState(ctx sdk.Context, height exported.Height, clientType string) (exported.ConsensusState, error) {
	if clientType != exported.Wasm {
		return k.clientKeeper.GetSelfConsensusState(ctx, height, clientType)
	}

	selfHeight, ok := height.(clienttypes.Height)
	if !ok {
		return nil, errorsmod.Wrapf(clienttypes.ErrInvalidHeight, "expected %T, got %T", clienttypes.Height{}, height)
	}
	// check that height revision matches chainID revision
	revision := clienttypes.ParseChainID(ctx.ChainID())
	if revision != height.GetRevisionNumber() {
		return nil, errorsmod.Wrapf(clienttypes.ErrInvalidHeight, "chainID revision number does not match height revision number: expected %d, got %d", revision, height.GetRevisionNumber())
	}

	histInfo, found := k.stakingKeeper.GetHistoricalInfo(ctx, int64(selfHeight.RevisionHeight))
	if !found {
		return nil, errorsmod.Wrapf(errors.ErrNotFound, "no historical info found at height %d", selfHeight.RevisionHeight)
	}

	timestamp := uint64(histInfo.Header.Time.Unix())

	cometblsConsensusState := &ConsensusState{
		Timestamp:          timestamp,
		Root:               commitmenttypes.NewMerkleRoot(histInfo.Header.GetAppHash()),
		NextValidatorsHash: histInfo.Header.NextValidatorsHash,
	}

	wasmData, err := k.cdc.Marshal(cometblsConsensusState)
	if err != nil {
		return nil, errorsmod.Wrapf(err, "cannot marshal cometbls consensus state")
	}

	consensusState := &wasmtypes.ConsensusState{
		Data:      wasmData,
		Timestamp: timestamp,
	}

	return consensusState, nil

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
