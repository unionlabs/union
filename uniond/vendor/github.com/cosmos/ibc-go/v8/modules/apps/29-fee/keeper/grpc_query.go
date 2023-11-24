package keeper

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/prefix"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
)

var _ types.QueryServer = (*Keeper)(nil)

// IncentivizedPackets implements the Query/IncentivizedPackets gRPC method
func (k Keeper) IncentivizedPackets(goCtx context.Context, req *types.QueryIncentivizedPacketsRequest) (*types.QueryIncentivizedPacketsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx).WithBlockHeight(int64(req.QueryHeight))

	var identifiedPackets []types.IdentifiedPacketFees
	store := prefix.NewStore(ctx.KVStore(k.storeKey), []byte(types.FeesInEscrowPrefix))
	pagination, err := query.Paginate(store, req.Pagination, func(key, value []byte) error {
		packetID, err := types.ParseKeyFeesInEscrow(types.FeesInEscrowPrefix + string(key))
		if err != nil {
			return err
		}

		packetFees := k.MustUnmarshalFees(value)
		identifiedPackets = append(identifiedPackets, types.NewIdentifiedPacketFees(packetID, packetFees.PacketFees))
		return nil
	})
	if err != nil {
		return nil, status.Error(codes.NotFound, err.Error())
	}

	return &types.QueryIncentivizedPacketsResponse{
		IncentivizedPackets: identifiedPackets,
		Pagination:          pagination,
	}, nil
}

// IncentivizedPacket implements the Query/IncentivizedPacket gRPC method
func (k Keeper) IncentivizedPacket(goCtx context.Context, req *types.QueryIncentivizedPacketRequest) (*types.QueryIncentivizedPacketResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx).WithBlockHeight(int64(req.QueryHeight))

	feesInEscrow, exists := k.GetFeesInEscrow(ctx, req.PacketId)
	if !exists {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrapf(types.ErrFeeNotFound, "channel: %s, port: %s, sequence: %d", req.PacketId.ChannelId, req.PacketId.PortId, req.PacketId.Sequence).Error())
	}

	return &types.QueryIncentivizedPacketResponse{
		IncentivizedPacket: types.NewIdentifiedPacketFees(req.PacketId, feesInEscrow.PacketFees),
	}, nil
}

// IncentivizedPacketsForChannel implements the Query/IncentivizedPacketsForChannel gRPC method
func (k Keeper) IncentivizedPacketsForChannel(goCtx context.Context, req *types.QueryIncentivizedPacketsForChannelRequest) (*types.QueryIncentivizedPacketsForChannelResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx).WithBlockHeight(int64(req.QueryHeight))

	var packets []*types.IdentifiedPacketFees
	keyPrefix := types.KeyFeesInEscrowChannelPrefix(req.PortId, req.ChannelId)
	store := prefix.NewStore(ctx.KVStore(k.storeKey), keyPrefix)
	pagination, err := query.Paginate(store, req.Pagination, func(key, value []byte) error {
		packetID, err := types.ParseKeyFeesInEscrow(string(keyPrefix) + string(key))
		if err != nil {
			return err
		}

		packetFees := k.MustUnmarshalFees(value)

		identifiedPacketFees := types.NewIdentifiedPacketFees(packetID, packetFees.PacketFees)
		packets = append(packets, &identifiedPacketFees)

		return nil
	})
	if err != nil {
		return nil, status.Error(codes.NotFound, err.Error())
	}

	return &types.QueryIncentivizedPacketsForChannelResponse{
		IncentivizedPackets: packets,
		Pagination:          pagination,
	}, nil
}

// TotalRecvFees implements the Query/TotalRecvFees gRPC method
func (k Keeper) TotalRecvFees(goCtx context.Context, req *types.QueryTotalRecvFeesRequest) (*types.QueryTotalRecvFeesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)

	feesInEscrow, found := k.GetFeesInEscrow(ctx, req.PacketId)
	if !found {
		return nil, status.Errorf(
			codes.NotFound,
			errorsmod.Wrapf(types.ErrFeeNotFound, "channel: %s, port: %s, sequence: %d", req.PacketId.ChannelId, req.PacketId.PortId, req.PacketId.Sequence).Error(),
		)
	}

	var recvFees sdk.Coins
	for _, packetFee := range feesInEscrow.PacketFees {
		recvFees = recvFees.Add(packetFee.Fee.RecvFee...)
	}

	return &types.QueryTotalRecvFeesResponse{
		RecvFees: recvFees,
	}, nil
}

// TotalAckFees implements the Query/TotalAckFees gRPC method
func (k Keeper) TotalAckFees(goCtx context.Context, req *types.QueryTotalAckFeesRequest) (*types.QueryTotalAckFeesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)

	feesInEscrow, found := k.GetFeesInEscrow(ctx, req.PacketId)
	if !found {
		return nil, status.Errorf(
			codes.NotFound,
			errorsmod.Wrapf(types.ErrFeeNotFound, "channel: %s, port: %s, sequence: %d", req.PacketId.ChannelId, req.PacketId.PortId, req.PacketId.Sequence).Error(),
		)
	}

	var ackFees sdk.Coins
	for _, packetFee := range feesInEscrow.PacketFees {
		ackFees = ackFees.Add(packetFee.Fee.AckFee...)
	}

	return &types.QueryTotalAckFeesResponse{
		AckFees: ackFees,
	}, nil
}

// TotalTimeoutFees implements the Query/TotalTimeoutFees gRPC method
func (k Keeper) TotalTimeoutFees(goCtx context.Context, req *types.QueryTotalTimeoutFeesRequest) (*types.QueryTotalTimeoutFeesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)

	feesInEscrow, found := k.GetFeesInEscrow(ctx, req.PacketId)
	if !found {
		return nil, status.Errorf(
			codes.NotFound,
			errorsmod.Wrapf(types.ErrFeeNotFound, "channel: %s, port: %s, sequence: %d", req.PacketId.ChannelId, req.PacketId.PortId, req.PacketId.Sequence).Error(),
		)
	}

	var timeoutFees sdk.Coins
	for _, packetFee := range feesInEscrow.PacketFees {
		timeoutFees = timeoutFees.Add(packetFee.Fee.TimeoutFee...)
	}

	return &types.QueryTotalTimeoutFeesResponse{
		TimeoutFees: timeoutFees,
	}, nil
}

// Payee implements the Query/Payee gRPC method and returns the registered payee address to which packet fees are paid out
func (k Keeper) Payee(goCtx context.Context, req *types.QueryPayeeRequest) (*types.QueryPayeeResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)

	payeeAddr, found := k.GetPayeeAddress(ctx, req.Relayer, req.ChannelId)
	if !found {
		return nil, status.Errorf(codes.NotFound, "payee address not found for address: %s on channel: %s", req.Relayer, req.ChannelId)
	}

	return &types.QueryPayeeResponse{
		PayeeAddress: payeeAddr,
	}, nil
}

// CounterpartyPayee implements the Query/CounterpartyPayee gRPC method and returns the registered counterparty payee address for forward relaying
func (k Keeper) CounterpartyPayee(goCtx context.Context, req *types.QueryCounterpartyPayeeRequest) (*types.QueryCounterpartyPayeeResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)

	counterpartyPayeeAddr, found := k.GetCounterpartyPayeeAddress(ctx, req.Relayer, req.ChannelId)
	if !found {
		return nil, status.Errorf(codes.NotFound, "counterparty payee address not found for address: %s on channel: %s", req.Relayer, req.ChannelId)
	}

	return &types.QueryCounterpartyPayeeResponse{
		CounterpartyPayee: counterpartyPayeeAddr,
	}, nil
}

// FeeEnabledChannels implements the Query/FeeEnabledChannels gRPC method and returns a list of fee enabled channels
func (k Keeper) FeeEnabledChannels(goCtx context.Context, req *types.QueryFeeEnabledChannelsRequest) (*types.QueryFeeEnabledChannelsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx).WithBlockHeight(int64(req.QueryHeight))

	var feeEnabledChannels []types.FeeEnabledChannel
	store := prefix.NewStore(ctx.KVStore(k.storeKey), []byte(types.FeeEnabledKeyPrefix))
	pagination, err := query.Paginate(store, req.Pagination, func(key, value []byte) error {
		portID, channelID, err := types.ParseKeyFeeEnabled(types.FeeEnabledKeyPrefix + string(key))
		if err != nil {
			return err
		}

		feeEnabledChannel := types.FeeEnabledChannel{
			PortId:    portID,
			ChannelId: channelID,
		}

		feeEnabledChannels = append(feeEnabledChannels, feeEnabledChannel)

		return nil
	})
	if err != nil {
		return nil, status.Error(codes.NotFound, err.Error())
	}

	return &types.QueryFeeEnabledChannelsResponse{
		FeeEnabledChannels: feeEnabledChannels,
		Pagination:         pagination,
	}, nil
}

// FeeEnabledChannel implements the Query/FeeEnabledChannel gRPC method and returns true if the provided
// port and channel identifiers belong to a fee enabled channel
func (k Keeper) FeeEnabledChannel(goCtx context.Context, req *types.QueryFeeEnabledChannelRequest) (*types.QueryFeeEnabledChannelResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(goCtx)

	isFeeEnabled := k.IsFeeEnabled(ctx, req.PortId, req.ChannelId)

	return &types.QueryFeeEnabledChannelResponse{
		FeeEnabled: isFeeEnabled,
	}, nil
}
