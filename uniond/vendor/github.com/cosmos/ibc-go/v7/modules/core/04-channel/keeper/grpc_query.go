package keeper

import (
	"context"
	"strconv"
	"strings"

	"github.com/cosmos/cosmos-sdk/store/prefix"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/query"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	clienttypes "github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v7/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v7/modules/core/24-host"
)

var _ types.QueryServer = (*Keeper)(nil)

// Channel implements the Query/Channel gRPC method
func (q Keeper) Channel(c context.Context, req *types.QueryChannelRequest) (*types.QueryChannelResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)
	channel, found := q.GetChannel(ctx, req.PortId, req.ChannelId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			sdkerrors.Wrapf(types.ErrChannelNotFound, "port-id: %s, channel-id %s", req.PortId, req.ChannelId).Error(),
		)
	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return types.NewQueryChannelResponse(channel, nil, selfHeight), nil
}

// Channels implements the Query/Channels gRPC method
func (q Keeper) Channels(c context.Context, req *types.QueryChannelsRequest) (*types.QueryChannelsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)

	channels := []*types.IdentifiedChannel{}
	store := prefix.NewStore(ctx.KVStore(q.storeKey), []byte(host.KeyChannelEndPrefix))

	pageRes, err := query.Paginate(store, req.Pagination, func(key, value []byte) error {
		var result types.Channel
		if err := q.cdc.Unmarshal(value, &result); err != nil {
			return err
		}

		portID, channelID, err := host.ParseChannelPath(string(key))
		if err != nil {
			return err
		}

		identifiedChannel := types.NewIdentifiedChannel(portID, channelID, result)
		channels = append(channels, &identifiedChannel)
		return nil
	})
	if err != nil {
		return nil, err
	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return &types.QueryChannelsResponse{
		Channels:   channels,
		Pagination: pageRes,
		Height:     selfHeight,
	}, nil
}

// ConnectionChannels implements the Query/ConnectionChannels gRPC method
func (q Keeper) ConnectionChannels(c context.Context, req *types.QueryConnectionChannelsRequest) (*types.QueryConnectionChannelsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := host.ConnectionIdentifierValidator(req.Connection); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)

	channels := []*types.IdentifiedChannel{}
	store := prefix.NewStore(ctx.KVStore(q.storeKey), []byte(host.KeyChannelEndPrefix))

	pageRes, err := query.FilteredPaginate(store, req.Pagination, func(key, value []byte, accumulate bool) (bool, error) {
		// filter any metadata stored under channel key
		var result types.Channel
		if err := q.cdc.Unmarshal(value, &result); err != nil {
			return false, err
		}

		// ignore channel and continue to the next item if the connection is
		// different than the requested one
		if result.ConnectionHops[0] != req.Connection {
			return false, nil
		}

		portID, channelID, err := host.ParseChannelPath(string(key))
		if err != nil {
			return false, err
		}

		identifiedChannel := types.NewIdentifiedChannel(portID, channelID, result)
		channels = append(channels, &identifiedChannel)
		return true, nil
	})
	if err != nil {
		return nil, err
	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return &types.QueryConnectionChannelsResponse{
		Channels:   channels,
		Pagination: pageRes,
		Height:     selfHeight,
	}, nil
}

// ChannelClientState implements the Query/ChannelClientState gRPC method
func (q Keeper) ChannelClientState(c context.Context, req *types.QueryChannelClientStateRequest) (*types.QueryChannelClientStateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	clientID, clientState, err := q.GetChannelClientState(ctx, req.PortId, req.ChannelId)
	if err != nil {
		return nil, status.Error(codes.NotFound, err.Error())
	}

	identifiedClientState := clienttypes.NewIdentifiedClientState(clientID, clientState)

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return types.NewQueryChannelClientStateResponse(identifiedClientState, nil, selfHeight), nil
}

// ChannelConsensusState implements the Query/ChannelConsensusState gRPC method
func (q Keeper) ChannelConsensusState(c context.Context, req *types.QueryChannelConsensusStateRequest) (*types.QueryChannelConsensusStateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	channel, found := q.GetChannel(ctx, req.PortId, req.ChannelId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			sdkerrors.Wrapf(types.ErrChannelNotFound, "port-id: %s, channel-id %s", req.PortId, req.ChannelId).Error(),
		)
	}

	connection, found := q.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return nil, status.Error(
			codes.NotFound,
			sdkerrors.Wrapf(connectiontypes.ErrConnectionNotFound, "connection-id: %s", channel.ConnectionHops[0]).Error(),
		)
	}

	consHeight := clienttypes.NewHeight(req.RevisionNumber, req.RevisionHeight)
	consensusState, found := q.clientKeeper.GetClientConsensusState(ctx, connection.ClientId, consHeight)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			sdkerrors.Wrapf(clienttypes.ErrConsensusStateNotFound, "client-id: %s", connection.ClientId).Error(),
		)
	}

	anyConsensusState, err := clienttypes.PackConsensusState(consensusState)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return types.NewQueryChannelConsensusStateResponse(connection.ClientId, anyConsensusState, consHeight, nil, selfHeight), nil
}

// PacketCommitment implements the Query/PacketCommitment gRPC method
func (q Keeper) PacketCommitment(c context.Context, req *types.QueryPacketCommitmentRequest) (*types.QueryPacketCommitmentResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	if req.Sequence == 0 {
		return nil, status.Error(codes.InvalidArgument, "packet sequence cannot be 0")
	}

	ctx := sdk.UnwrapSDKContext(c)

	commitmentBz := q.GetPacketCommitment(ctx, req.PortId, req.ChannelId, req.Sequence)
	if len(commitmentBz) == 0 {
		return nil, status.Error(codes.NotFound, "packet commitment hash not found")
	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return types.NewQueryPacketCommitmentResponse(commitmentBz, nil, selfHeight), nil
}

// PacketCommitments implements the Query/PacketCommitments gRPC method
func (q Keeper) PacketCommitments(c context.Context, req *types.QueryPacketCommitmentsRequest) (*types.QueryPacketCommitmentsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	commitments := []*types.PacketState{}
	store := prefix.NewStore(ctx.KVStore(q.storeKey), []byte(host.PacketCommitmentPrefixPath(req.PortId, req.ChannelId)))

	pageRes, err := query.Paginate(store, req.Pagination, func(key, value []byte) error {
		keySplit := strings.Split(string(key), "/")

		sequence, err := strconv.ParseUint(keySplit[len(keySplit)-1], 10, 64)
		if err != nil {
			return err
		}

		commitment := types.NewPacketState(req.PortId, req.ChannelId, sequence, value)
		commitments = append(commitments, &commitment)
		return nil
	})
	if err != nil {
		return nil, err
	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return &types.QueryPacketCommitmentsResponse{
		Commitments: commitments,
		Pagination:  pageRes,
		Height:      selfHeight,
	}, nil
}

// PacketReceipt implements the Query/PacketReceipt gRPC method
func (q Keeper) PacketReceipt(c context.Context, req *types.QueryPacketReceiptRequest) (*types.QueryPacketReceiptResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	if req.Sequence == 0 {
		return nil, status.Error(codes.InvalidArgument, "packet sequence cannot be 0")
	}

	ctx := sdk.UnwrapSDKContext(c)

	_, recvd := q.GetPacketReceipt(ctx, req.PortId, req.ChannelId, req.Sequence)

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return types.NewQueryPacketReceiptResponse(recvd, nil, selfHeight), nil
}

// PacketAcknowledgement implements the Query/PacketAcknowledgement gRPC method
func (q Keeper) PacketAcknowledgement(c context.Context, req *types.QueryPacketAcknowledgementRequest) (*types.QueryPacketAcknowledgementResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	if req.Sequence == 0 {
		return nil, status.Error(codes.InvalidArgument, "packet sequence cannot be 0")
	}

	ctx := sdk.UnwrapSDKContext(c)

	acknowledgementBz, found := q.GetPacketAcknowledgement(ctx, req.PortId, req.ChannelId, req.Sequence)
	if !found || len(acknowledgementBz) == 0 {
		return nil, status.Error(codes.NotFound, "packet acknowledgement hash not found")
	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return types.NewQueryPacketAcknowledgementResponse(acknowledgementBz, nil, selfHeight), nil
}

// PacketAcknowledgements implements the Query/PacketAcknowledgements gRPC method
func (q Keeper) PacketAcknowledgements(c context.Context, req *types.QueryPacketAcknowledgementsRequest) (*types.QueryPacketAcknowledgementsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	acks := []*types.PacketState{}
	store := prefix.NewStore(ctx.KVStore(q.storeKey), []byte(host.PacketAcknowledgementPrefixPath(req.PortId, req.ChannelId)))

	// if a list of packet sequences is provided then query for each specific ack and return a list <= len(req.PacketCommitmentSequences)
	// otherwise, maintain previous behaviour and perform paginated query
	for _, seq := range req.PacketCommitmentSequences {
		acknowledgementBz, found := q.GetPacketAcknowledgement(ctx, req.PortId, req.ChannelId, seq)
		if !found || len(acknowledgementBz) == 0 {
			continue
		}

		ack := types.NewPacketState(req.PortId, req.ChannelId, seq, acknowledgementBz)
		acks = append(acks, &ack)
	}

	if len(req.PacketCommitmentSequences) > 0 {
		selfHeight := clienttypes.GetSelfHeight(ctx)
		return &types.QueryPacketAcknowledgementsResponse{
			Acknowledgements: acks,
			Pagination:       nil,
			Height:           selfHeight,
		}, nil
	}

	pageRes, err := query.Paginate(store, req.Pagination, func(key, value []byte) error {
		keySplit := strings.Split(string(key), "/")

		sequence, err := strconv.ParseUint(keySplit[len(keySplit)-1], 10, 64)
		if err != nil {
			return err
		}

		ack := types.NewPacketState(req.PortId, req.ChannelId, sequence, value)
		acks = append(acks, &ack)

		return nil
	})
	if err != nil {
		return nil, err
	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return &types.QueryPacketAcknowledgementsResponse{
		Acknowledgements: acks,
		Pagination:       pageRes,
		Height:           selfHeight,
	}, nil
}

// UnreceivedPackets implements the Query/UnreceivedPackets gRPC method. Given
// a list of counterparty packet commitments, the querier checks if the packet
// has already been received by checking if a receipt exists on this
// chain for the packet sequence. All packets that haven't been received yet
// are returned in the response
// Usage: To use this method correctly, first query all packet commitments on
// the sending chain using the Query/PacketCommitments gRPC method.
// Then input the returned sequences into the QueryUnreceivedPacketsRequest
// and send the request to this Query/UnreceivedPackets on the **receiving**
// chain. This gRPC method will then return the list of packet sequences that
// are yet to be received on the receiving chain.
//
// NOTE: The querier makes the assumption that the provided list of packet
// commitments is correct and will not function properly if the list
// is not up to date. Ideally the query height should equal the latest height
// on the counterparty's client which represents this chain.
func (q Keeper) UnreceivedPackets(c context.Context, req *types.QueryUnreceivedPacketsRequest) (*types.QueryUnreceivedPacketsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	unreceivedSequences := []uint64{}

	for i, seq := range req.PacketCommitmentSequences {
		if seq == 0 {
			return nil, status.Errorf(codes.InvalidArgument, "packet sequence %d cannot be 0", i)
		}

		// if packet receipt exists on the receiving chain, then packet has already been received
		if _, found := q.GetPacketReceipt(ctx, req.PortId, req.ChannelId, seq); !found {
			unreceivedSequences = append(unreceivedSequences, seq)
		}

	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return &types.QueryUnreceivedPacketsResponse{
		Sequences: unreceivedSequences,
		Height:    selfHeight,
	}, nil
}

// UnreceivedAcks implements the Query/UnreceivedAcks gRPC method. Given
// a list of counterparty packet acknowledgements, the querier checks if the packet
// has already been received by checking if the packet commitment still exists on this
// chain (original sender) for the packet sequence.
// All acknowledgmeents that haven't been received yet are returned in the response.
// Usage: To use this method correctly, first query all packet acknowledgements on
// the original receiving chain (ie the chain that wrote the acks) using the Query/PacketAcknowledgements gRPC method.
// Then input the returned sequences into the QueryUnreceivedAcksRequest
// and send the request to this Query/UnreceivedAcks on the **original sending**
// chain. This gRPC method will then return the list of packet sequences whose
// acknowledgements are already written on the receiving chain but haven't yet
// been received back to the sending chain.
//
// NOTE: The querier makes the assumption that the provided list of packet
// acknowledgements is correct and will not function properly if the list
// is not up to date. Ideally the query height should equal the latest height
// on the counterparty's client which represents this chain.
func (q Keeper) UnreceivedAcks(c context.Context, req *types.QueryUnreceivedAcksRequest) (*types.QueryUnreceivedAcksResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	unreceivedSequences := []uint64{}

	for i, seq := range req.PacketAckSequences {
		if seq == 0 {
			return nil, status.Errorf(codes.InvalidArgument, "packet sequence %d cannot be 0", i)
		}

		// if packet commitment still exists on the original sending chain, then packet ack has not been received
		// since processing the ack will delete the packet commitment
		if commitment := q.GetPacketCommitment(ctx, req.PortId, req.ChannelId, seq); len(commitment) != 0 {
			unreceivedSequences = append(unreceivedSequences, seq)
		}

	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return &types.QueryUnreceivedAcksResponse{
		Sequences: unreceivedSequences,
		Height:    selfHeight,
	}, nil
}

// NextSequenceReceive implements the Query/NextSequenceReceive gRPC method
func (q Keeper) NextSequenceReceive(c context.Context, req *types.QueryNextSequenceReceiveRequest) (*types.QueryNextSequenceReceiveResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := validategRPCRequest(req.PortId, req.ChannelId); err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)
	sequence, found := q.GetNextSequenceRecv(ctx, req.PortId, req.ChannelId)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			sdkerrors.Wrapf(types.ErrSequenceReceiveNotFound, "port-id: %s, channel-id %s", req.PortId, req.ChannelId).Error(),
		)
	}

	selfHeight := clienttypes.GetSelfHeight(ctx)
	return types.NewQueryNextSequenceReceiveResponse(sequence, nil, selfHeight), nil
}

func validategRPCRequest(portID, channelID string) error {
	if err := host.PortIdentifierValidator(portID); err != nil {
		return status.Error(codes.InvalidArgument, err.Error())
	}

	if err := host.ChannelIdentifierValidator(channelID); err != nil {
		return status.Error(codes.InvalidArgument, err.Error())
	}

	return nil
}
