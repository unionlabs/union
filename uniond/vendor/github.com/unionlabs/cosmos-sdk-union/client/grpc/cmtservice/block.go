package cmtservice

import (
	"context"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
)

func getBlockHeight(ctx context.Context, rpc CometRPC) (int64, error) {
	status, err := GetNodeStatus(ctx, rpc)
	if err != nil {
		return 0, err
	}
	height := status.SyncInfo.LatestBlockHeight
	return height, nil
}

func getBlock(ctx context.Context, rpc CometRPC, height *int64) (*coretypes.ResultBlock, error) {
	return rpc.Block(ctx, height)
}

func GetProtoBlock(ctx context.Context, rpc CometRPC, height *int64) (cmtproto.BlockID, *cmtproto.Block, error) {
	block, err := getBlock(ctx, rpc, height)
	if err != nil {
		return cmtproto.BlockID{}, nil, err
	}
	protoBlock, err := block.Block.ToProto()
	if err != nil {
		return cmtproto.BlockID{}, nil, err
	}
	protoBlockID := block.BlockID.ToProto()

	return protoBlockID, protoBlock, nil
}
