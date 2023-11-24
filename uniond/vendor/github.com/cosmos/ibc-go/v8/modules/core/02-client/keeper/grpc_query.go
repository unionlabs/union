package keeper

import (
	"bytes"
	"context"
	"fmt"
	"sort"
	"strings"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/prefix"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ types.QueryServer = (*Keeper)(nil)

// ClientState implements the Query/ClientState gRPC method
func (k Keeper) ClientState(c context.Context, req *types.QueryClientStateRequest) (*types.QueryClientStateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ClientIdentifierValidator(req.ClientId); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)
	clientState, found := k.GetClientState(ctx, req.ClientId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrap(types.ErrClientNotFound, req.ClientId).Error(),
		)
	}

	protoAny, err := types.PackClientState(clientState)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	proofHeight := types.GetSelfHeight(ctx)
	return &types.QueryClientStateResponse{
		ClientState: protoAny,
		ProofHeight: proofHeight,
	}, nil
}

// ClientStates implements the Query/ClientStates gRPC method
func (k Keeper) ClientStates(c context.Context, req *types.QueryClientStatesRequest) (*types.QueryClientStatesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)

	var clientStates types.IdentifiedClientStates
	store := prefix.NewStore(ctx.KVStore(k.storeKey), host.KeyClientStorePrefix)

	pageRes, err := query.FilteredPaginate(store, req.Pagination, func(key, value []byte, accumulate bool) (bool, error) {
		// filter any metadata stored under client state key
		keySplit := strings.Split(string(key), "/")
		if keySplit[len(keySplit)-1] != "clientState" {
			return false, nil
		}

		clientState, err := k.UnmarshalClientState(value)
		if err != nil {
			return false, err
		}

		clientID := keySplit[1]
		if err := host.ClientIdentifierValidator(clientID); err != nil {
			return false, err
		}

		identifiedClient := types.NewIdentifiedClientState(clientID, clientState)
		clientStates = append(clientStates, identifiedClient)
		return true, nil
	})
	if err != nil {
		return nil, err
	}

	sort.Sort(clientStates)

	return &types.QueryClientStatesResponse{
		ClientStates: clientStates,
		Pagination:   pageRes,
	}, nil
}

// ConsensusState implements the Query/ConsensusState gRPC method
func (k Keeper) ConsensusState(c context.Context, req *types.QueryConsensusStateRequest) (*types.QueryConsensusStateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ClientIdentifierValidator(req.ClientId); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)

	var (
		consensusState exported.ConsensusState
		found          bool
	)

	height := types.NewHeight(req.RevisionNumber, req.RevisionHeight)
	if req.LatestHeight {
		consensusState, found = k.GetLatestClientConsensusState(ctx, req.ClientId)
	} else {
		if req.RevisionHeight == 0 {
			return nil, status.Error(codes.InvalidArgument, "consensus state height cannot be 0")
		}

		consensusState, found = k.GetClientConsensusState(ctx, req.ClientId, height)
	}

	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrapf(types.ErrConsensusStateNotFound, "client-id: %s, height: %s", req.ClientId, height).Error(),
		)
	}

	protoAny, err := types.PackConsensusState(consensusState)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	proofHeight := types.GetSelfHeight(ctx)
	return &types.QueryConsensusStateResponse{
		ConsensusState: protoAny,
		ProofHeight:    proofHeight,
	}, nil
}

// ConsensusStates implements the Query/ConsensusStates gRPC method
func (k Keeper) ConsensusStates(c context.Context, req *types.QueryConsensusStatesRequest) (*types.QueryConsensusStatesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ClientIdentifierValidator(req.ClientId); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)

	var consensusStates []types.ConsensusStateWithHeight
	store := prefix.NewStore(ctx.KVStore(k.storeKey), host.FullClientKey(req.ClientId, []byte(fmt.Sprintf("%s/", host.KeyConsensusStatePrefix))))

	pageRes, err := query.FilteredPaginate(store, req.Pagination, func(key, value []byte, accumulate bool) (bool, error) {
		// filter any metadata stored under consensus state key
		if bytes.Contains(key, []byte("/")) {
			return false, nil
		}

		height, err := types.ParseHeight(string(key))
		if err != nil {
			return false, err
		}

		consensusState, err := k.UnmarshalConsensusState(value)
		if err != nil {
			return false, err
		}

		consensusStates = append(consensusStates, types.NewConsensusStateWithHeight(height, consensusState))
		return true, nil
	})
	if err != nil {
		return nil, err
	}

	return &types.QueryConsensusStatesResponse{
		ConsensusStates: consensusStates,
		Pagination:      pageRes,
	}, nil
}

// ConsensusStateHeights implements the Query/ConsensusStateHeights gRPC method
func (k Keeper) ConsensusStateHeights(c context.Context, req *types.QueryConsensusStateHeightsRequest) (*types.QueryConsensusStateHeightsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ClientIdentifierValidator(req.ClientId); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)

	var consensusStateHeights []types.Height
	store := prefix.NewStore(ctx.KVStore(k.storeKey), host.FullClientKey(req.ClientId, []byte(fmt.Sprintf("%s/", host.KeyConsensusStatePrefix))))

	pageRes, err := query.FilteredPaginate(store, req.Pagination, func(key, _ []byte, accumulate bool) (bool, error) {
		// filter any metadata stored under consensus state key
		if bytes.Contains(key, []byte("/")) {
			return false, nil
		}

		height, err := types.ParseHeight(string(key))
		if err != nil {
			return false, err
		}

		consensusStateHeights = append(consensusStateHeights, height)
		return true, nil
	})
	if err != nil {
		return nil, err
	}

	return &types.QueryConsensusStateHeightsResponse{
		ConsensusStateHeights: consensusStateHeights,
		Pagination:            pageRes,
	}, nil
}

// ClientStatus implements the Query/ClientStatus gRPC method
func (k Keeper) ClientStatus(c context.Context, req *types.QueryClientStatusRequest) (*types.QueryClientStatusResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ClientIdentifierValidator(req.ClientId); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)
	clientState, found := k.GetClientState(ctx, req.ClientId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrap(types.ErrClientNotFound, req.ClientId).Error(),
		)
	}

	clientStatus := k.GetClientStatus(ctx, clientState, req.ClientId)

	return &types.QueryClientStatusResponse{
		Status: clientStatus.String(),
	}, nil
}

// ClientParams implements the Query/ClientParams gRPC method
func (k Keeper) ClientParams(c context.Context, _ *types.QueryClientParamsRequest) (*types.QueryClientParamsResponse, error) {
	ctx := sdk.UnwrapSDKContext(c)
	params := k.GetParams(ctx)

	return &types.QueryClientParamsResponse{
		Params: &params,
	}, nil
}

// UpgradedClientState implements the Query/UpgradedClientState gRPC method
func (k Keeper) UpgradedClientState(c context.Context, req *types.QueryUpgradedClientStateRequest) (*types.QueryUpgradedClientStateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)

	plan, err := k.GetUpgradePlan(ctx)
	if err != nil {
		return nil, status.Error(codes.NotFound, err.Error())
	}

	bz, err := k.GetUpgradedClient(ctx, plan.Height)
	if err != nil {
		return nil, status.Error(codes.NotFound, err.Error())
	}

	clientState, err := types.UnmarshalClientState(k.cdc, bz)
	if err != nil {
		return nil, status.Error(
			codes.Internal, err.Error(),
		)
	}

	protoAny, err := types.PackClientState(clientState)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryUpgradedClientStateResponse{
		UpgradedClientState: protoAny,
	}, nil
}

// UpgradedConsensusState implements the Query/UpgradedConsensusState gRPC method
func (k Keeper) UpgradedConsensusState(c context.Context, req *types.QueryUpgradedConsensusStateRequest) (*types.QueryUpgradedConsensusStateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)

	bz, err := k.GetUpgradedConsensusState(ctx, ctx.BlockHeight())
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "%s, height %d", err.Error(), ctx.BlockHeight())
	}

	consensusState, err := types.UnmarshalConsensusState(k.cdc, bz)
	if err != nil {
		return nil, status.Error(
			codes.Internal, err.Error(),
		)
	}

	protoAny, err := types.PackConsensusState(consensusState)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryUpgradedConsensusStateResponse{
		UpgradedConsensusState: protoAny,
	}, nil
}
