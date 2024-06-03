package keeper

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"union/x/deferredack/types"
)

type msgServer struct {
	Keeper
}

func NewMsgServerImpl(keeper Keeper) types.MsgServer {
	return msgServer{Keeper: keeper}
}

var _ types.MsgServer = msgServer{}

func (server msgServer) WriteDeferredAck(goCtx context.Context, req *types.MsgWriteDeferredAck) (*types.MsgWriteDeferredAckResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	err := server.Keeper.WriteDeferredAck(ctx, req.DeferredPacketInfo, *req.Ack)

	if err != nil {
		return nil, err
	}

	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.TypeMsgWriteDeferredAck,
		),
	})

	return &types.MsgWriteDeferredAckResponse{}, nil
}
