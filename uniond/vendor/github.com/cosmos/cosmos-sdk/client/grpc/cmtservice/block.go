package cmtservice

import (
	"context"

	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"

	"github.com/cosmos/cosmos-sdk/client"
)

func getBlockHeight(ctx context.Context, clientCtx client.Context) (int64, error) {
	status, err := GetNodeStatus(ctx, clientCtx)
	if err != nil {
		return 0, err
	}
	height := status.SyncInfo.LatestBlockHeight
	return height, nil
}

func getBlock(ctx context.Context, clientCtx client.Context, height *int64) (*coretypes.ResultBlock, error) {
	// get the node
	node, err := clientCtx.GetNode()
	if err != nil {
		return nil, err
	}

	return node.Block(ctx, height)
}

func GetProtoBlock(ctx context.Context, clientCtx client.Context, height *int64) (cmtproto.BlockID, *cmtproto.Block, error) {
	block, err := getBlock(ctx, clientCtx, height)
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
