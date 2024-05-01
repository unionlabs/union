package keeper

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"union/x/differedack/types"
)

type msgServer struct {
	Keeper
}

func NewMsgServerImpl(keeper Keeper) types.MsgServer {
	return msgServer{Keeper: keeper}
}

var _ types.MsgServer = msgServer{}

func (server msgServer) WriteDifferedAck(goCtx context.Context, req *types.MsgWriteDifferedAck) (*types.MsgWriteDifferedAckResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	err := server.Keeper.WriteDifferedAck(ctx, *req.Packet, *req.Data, req.DifferedPacketInfo, *req.Ack)

	if err != nil {
		return nil, err
	}

	ctx.EventManager().EmitEvent(sdk.Events{
		sdk.NewEvent()
	})

	return &types.MsgWriteDifferedAckResponse{}, nil
}
