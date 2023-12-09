package keeper

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/prefix"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

var _ types.QueryServer = (*Keeper)(nil)

// Connection implements the Query/Connection gRPC method
func (k Keeper) Connection(c context.Context, req *types.QueryConnectionRequest) (*types.QueryConnectionResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ConnectionIdentifierValidator(req.ConnectionId); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)
	connection, found := k.GetConnection(ctx, req.ConnectionId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrap(types.ErrConnectionNotFound, req.ConnectionId).Error(),
		)
	}

	return &types.QueryConnectionResponse{
		Connection:  &connection,
		ProofHeight: clienttypes.GetSelfHeight(ctx),
	}, nil
}

// Connections implements the Query/Connections gRPC method
func (k Keeper) Connections(c context.Context, req *types.QueryConnectionsRequest) (*types.QueryConnectionsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)

	var connections []*types.IdentifiedConnection
	store := prefix.NewStore(ctx.KVStore(k.storeKey), []byte(host.KeyConnectionPrefix))

	pageRes, err := query.Paginate(store, req.Pagination, func(key, value []byte) error {
		var result types.ConnectionEnd
		if err := k.cdc.Unmarshal(value, &result); err != nil {
			return err
		}

		connectionID, err := host.ParseConnectionPath(string(key))
		if err != nil {
			return err
		}

		identifiedConnection := types.NewIdentifiedConnection(connectionID, result)
		connections = append(connections, &identifiedConnection)
		return nil
	})
	if err != nil {
		return nil, err
	}

	return &types.QueryConnectionsResponse{
		Connections: connections,
		Pagination:  pageRes,
		Height:      clienttypes.GetSelfHeight(ctx),
	}, nil
}

// ClientConnections implements the Query/ClientConnections gRPC method
func (k Keeper) ClientConnections(c context.Context, req *types.QueryClientConnectionsRequest) (*types.QueryClientConnectionsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ClientIdentifierValidator(req.ClientId); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)
	clientConnectionPaths, found := k.GetClientConnectionPaths(ctx, req.ClientId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrap(types.ErrClientConnectionPathsNotFound, req.ClientId).Error(),
		)
	}

	return &types.QueryClientConnectionsResponse{
		ConnectionPaths: clientConnectionPaths,
		ProofHeight:     clienttypes.GetSelfHeight(ctx),
	}, nil
}

// ConnectionClientState implements the Query/ConnectionClientState gRPC method
func (k Keeper) ConnectionClientState(c context.Context, req *types.QueryConnectionClientStateRequest) (*types.QueryConnectionClientStateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ConnectionIdentifierValidator(req.ConnectionId); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)

	connection, found := k.GetConnection(ctx, req.ConnectionId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrapf(types.ErrConnectionNotFound, "connection-id: %s", req.ConnectionId).Error(),
		)
	}

	clientState, found := k.clientKeeper.GetClientState(ctx, connection.ClientId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrapf(clienttypes.ErrClientNotFound, "client-id: %s", connection.ClientId).Error(),
		)
	}

	identifiedClientState := clienttypes.NewIdentifiedClientState(connection.ClientId, clientState)

	height := clienttypes.GetSelfHeight(ctx)
	return types.NewQueryConnectionClientStateResponse(identifiedClientState, nil, height), nil
}

// ConnectionConsensusState implements the Query/ConnectionConsensusState gRPC method
func (k Keeper) ConnectionConsensusState(c context.Context, req *types.QueryConnectionConsensusStateRequest) (*types.QueryConnectionConsensusStateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ConnectionIdentifierValidator(req.ConnectionId); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)

	connection, found := k.GetConnection(ctx, req.ConnectionId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrapf(types.ErrConnectionNotFound, "connection-id: %s", req.ConnectionId).Error(),
		)
	}

	height := clienttypes.NewHeight(req.RevisionNumber, req.RevisionHeight)
	consensusState, found := k.clientKeeper.GetClientConsensusState(ctx, connection.ClientId, height)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrapf(clienttypes.ErrConsensusStateNotFound, "client-id: %s", connection.ClientId).Error(),
		)
	}

	anyConsensusState, err := clienttypes.PackConsensusState(consensusState)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	proofHeight := clienttypes.GetSelfHeight(ctx)
	return types.NewQueryConnectionConsensusStateResponse(connection.ClientId, anyConsensusState, height, nil, proofHeight), nil
}

// ConnectionParams implements the Query/ConnectionParams gRPC method.
func (k Keeper) ConnectionParams(c context.Context, req *types.QueryConnectionParamsRequest) (*types.QueryConnectionParamsResponse, error) {
	ctx := sdk.UnwrapSDKContext(c)
	params := k.GetParams(ctx)

	return &types.QueryConnectionParamsResponse{
		Params: &params,
	}, nil
}
