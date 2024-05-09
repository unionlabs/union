package keeper

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"union/x/diferredack/types"
)

type msgServer struct {
	Keeper
}

func NewMsgServerImpl(keeper Keeper) types.MsgServer {
	return msgServer{Keeper: keeper}
}

var _ types.MsgServer = msgServer{}

func (server msgServer) WriteDiferredAck(goCtx context.Context, req *types.MsgWriteDiferredAck) (*types.MsgWriteDiferredAckResponse, error) {
	ctx := sdk.UnwrapSDKContext(goCtx)

	err := server.Keeper.WriteDiferredAck(ctx, *req.Packet, *req.Data, req.DiferredPacketInfo, *req.Ack)

	if err != nil {
		return nil, err
	}

	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.TypeMsgWriteDiferredAck,
		),
	})

	return &types.MsgWriteDiferredAckResponse{}, nil
}
