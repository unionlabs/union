package utils

import (
	"context"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"

	tmtypes "github.com/cometbft/cometbft/types"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcclient "github.com/cosmos/ibc-go/v8/modules/core/client"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
)

// QueryClientState returns a client state. If prove is true, it performs an ABCI store query
// in order to retrieve the merkle proof. Otherwise, it uses the gRPC query client.
func QueryClientState(
	clientCtx client.Context, clientID string, prove bool,
) (*types.QueryClientStateResponse, error) {
	if prove {
		return QueryClientStateABCI(clientCtx, clientID)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryClientStateRequest{
		ClientId: clientID,
	}

	return queryClient.ClientState(context.Background(), req)
}

// QueryClientStateABCI queries the store to get the light client state and a merkle proof.
func QueryClientStateABCI(
	clientCtx client.Context, clientID string,
) (*types.QueryClientStateResponse, error) {
	key := host.FullClientStateKey(clientID)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if client exists
	if len(value) == 0 {
		return nil, errorsmod.Wrap(types.ErrClientNotFound, clientID)
	}

	cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

	clientState, err := types.UnmarshalClientState(cdc, value)
	if err != nil {
		return nil, err
	}

	anyClientState, err := types.PackClientState(clientState)
	if err != nil {
		return nil, err
	}

	clientStateRes := types.NewQueryClientStateResponse(anyClientState, proofBz, proofHeight)
	return clientStateRes, nil
}

// QueryConsensusState returns a consensus state. If prove is true, it performs an ABCI store
// query in order to retrieve the merkle proof. Otherwise, it uses the gRPC query client.
func QueryConsensusState(
	clientCtx client.Context, clientID string, height exported.Height, prove, latestHeight bool,
) (*types.QueryConsensusStateResponse, error) {
	if prove {
		return QueryConsensusStateABCI(clientCtx, clientID, height)
	}

	queryClient := types.NewQueryClient(clientCtx)
	req := &types.QueryConsensusStateRequest{
		ClientId:       clientID,
		RevisionNumber: height.GetRevisionNumber(),
		RevisionHeight: height.GetRevisionHeight(),
		LatestHeight:   latestHeight,
	}

	return queryClient.ConsensusState(context.Background(), req)
}

// QueryConsensusStateABCI queries the store to get the consensus state of a light client and a
// merkle proof of its existence or non-existence.
func QueryConsensusStateABCI(
	clientCtx client.Context, clientID string, height exported.Height,
) (*types.QueryConsensusStateResponse, error) {
	key := host.FullConsensusStateKey(clientID, height)

	value, proofBz, proofHeight, err := ibcclient.QueryTendermintProof(clientCtx, key)
	if err != nil {
		return nil, err
	}

	// check if consensus state exists
	if len(value) == 0 {
		return nil, errorsmod.Wrap(types.ErrConsensusStateNotFound, clientID)
	}

	cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

	cs, err := types.UnmarshalConsensusState(cdc, value)
	if err != nil {
		return nil, err
	}

	anyConsensusState, err := types.PackConsensusState(cs)
	if err != nil {
		return nil, err
	}

	return types.NewQueryConsensusStateResponse(anyConsensusState, proofBz, proofHeight), nil
}

// QueryTendermintHeader takes a client context and returns the appropriate
// tendermint header
func QueryTendermintHeader(clientCtx client.Context) (ibctm.Header, int64, error) {
	node, err := clientCtx.GetNode()
	if err != nil {
		return ibctm.Header{}, 0, err
	}

	info, err := node.ABCIInfo(context.Background())
	if err != nil {
		return ibctm.Header{}, 0, err
	}

	var height int64
	if clientCtx.Height != 0 {
		height = clientCtx.Height
	} else {
		height = info.Response.LastBlockHeight
	}

	commit, err := node.Commit(context.Background(), &height)
	if err != nil {
		return ibctm.Header{}, 0, err
	}

	page := 1
	count := 10_000

	validators, err := node.Validators(context.Background(), &height, &page, &count)
	if err != nil {
		return ibctm.Header{}, 0, err
	}

	protoCommit := commit.SignedHeader.ToProto()
	protoValset, err := tmtypes.NewValidatorSet(validators.Validators).ToProto()
	if err != nil {
		return ibctm.Header{}, 0, err
	}

	header := ibctm.Header{
		SignedHeader: protoCommit,
		ValidatorSet: protoValset,
	}

	return header, height, nil
}

// QuerySelfConsensusState takes a client context and returns the appropriate
// tendermint consensus state
func QuerySelfConsensusState(clientCtx client.Context) (*ibctm.ConsensusState, int64, error) {
	node, err := clientCtx.GetNode()
	if err != nil {
		return &ibctm.ConsensusState{}, 0, err
	}

	info, err := node.ABCIInfo(context.Background())
	if err != nil {
		return &ibctm.ConsensusState{}, 0, err
	}

	var height int64
	if clientCtx.Height != 0 {
		height = clientCtx.Height
	} else {
		height = info.Response.LastBlockHeight
	}

	commit, err := node.Commit(context.Background(), &height)
	if err != nil {
		return &ibctm.ConsensusState{}, 0, err
	}

	page := 1
	count := 10_000

	nextHeight := height + 1
	nextVals, err := node.Validators(context.Background(), &nextHeight, &page, &count)
	if err != nil {
		return &ibctm.ConsensusState{}, 0, err
	}

	state := &ibctm.ConsensusState{
		Timestamp:          commit.Time,
		Root:               commitmenttypes.NewMerkleRoot(commit.AppHash),
		NextValidatorsHash: tmtypes.NewValidatorSet(nextVals.Validators).Hash(),
	}

	return state, height, nil
}
