package client

import (
	"context"
	"fmt"

	"github.com/cosmos/gogoproto/grpc"

	blocksvc "github.com/cometbft/cometbft/api/cometbft/services/block/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/types"
)

// Block data returned by the CometBFT BlockService gRPC API.
type Block struct {
	BlockID *types.BlockID `json:"block_id"`
	Block   *types.Block   `json:"block"`
}

func blockFromProto(pblockID *cmtproto.BlockID, pblock *cmtproto.Block) (*Block, error) {
	blockID, err := types.BlockIDFromProto(pblockID)
	if err != nil {
		return nil, err
	}

	block, err := types.BlockFromProto(pblock)
	if err != nil {
		return nil, err
	}

	return &Block{
		BlockID: blockID,
		Block:   block,
	}, nil
}

// LatestHeightResult type used in GetLatestResult and send to the client
// via a channel.
type LatestHeightResult struct {
	Height int64
	Error  error
}

type getLatestHeightConfig struct {
	chSize uint
}

type GetLatestHeightOption func(*getLatestHeightConfig)

// GetLatestHeightChannelSize allows control over the channel size. If not used
// or the channel size is set to 0, an unbuffered channel will be created.
func GetLatestHeightChannelSize(sz uint) GetLatestHeightOption {
	return func(opts *getLatestHeightConfig) {
		opts.chSize = sz
	}
}

// BlockServiceClient provides block information.
type BlockServiceClient interface {
	// GetBlockByHeight attempts to retrieve the block associated with the
	// given height.
	GetBlockByHeight(ctx context.Context, height int64) (*Block, error)

	// GetLatestHeight provides sends the latest committed block height to the
	// resulting output channel as blocks are committed.
	GetLatestHeight(ctx context.Context, opts ...GetLatestHeightOption) (<-chan LatestHeightResult, error)
}

type blockServiceClient struct {
	client blocksvc.BlockServiceClient
}

func newBlockServiceClient(conn grpc.ClientConn) BlockServiceClient {
	return &blockServiceClient{
		client: blocksvc.NewBlockServiceClient(conn),
	}
}

// GetBlockByHeight implements BlockServiceClient GetBlockByHeight.
func (c *blockServiceClient) GetBlockByHeight(ctx context.Context, height int64) (*Block, error) {
	res, err := c.client.GetByHeight(ctx, &blocksvc.GetByHeightRequest{
		Height: height,
	})
	if err != nil {
		return nil, err
	}

	return blockFromProto(res.BlockId, res.Block)
}

// GetLatestHeight implements BlockServiceClient GetLatestHeight.
func (c *blockServiceClient) GetLatestHeight(ctx context.Context, opts ...GetLatestHeightOption) (<-chan LatestHeightResult, error) {
	req := blocksvc.GetLatestHeightRequest{}

	latestHeightClient, err := c.client.GetLatestHeight(ctx, &req)
	if err != nil {
		return nil, fmt.Errorf("error getting a stream for the latest height: %w", err)
	}

	cfg := &getLatestHeightConfig{}
	for _, opt := range opts {
		opt(cfg)
	}
	resultCh := make(chan LatestHeightResult, cfg.chSize)

	go func(client blocksvc.BlockService_GetLatestHeightClient) {
		defer close(resultCh)
		for {
			response, err := client.Recv()
			if err != nil {
				res := LatestHeightResult{Error: fmt.Errorf("error receiving the latest height from a stream: %w", err)}
				select {
				case <-ctx.Done():
				case resultCh <- res:
				}
				return
			}
			res := LatestHeightResult{Height: response.Height}
			select {
			case <-ctx.Done():
				return
			case resultCh <- res:
			default:
				// Skip sending this result because the channel is full - the
				// client will get the next one once the channel opens up again
			}
		}
	}(latestHeightClient)

	return resultCh, nil
}

type disabledBlockServiceClient struct{}

func newDisabledBlockServiceClient() BlockServiceClient {
	return &disabledBlockServiceClient{}
}

// GetBlockByHeight implements BlockServiceClient GetBlockByHeight - disabled client.
func (*disabledBlockServiceClient) GetBlockByHeight(context.Context, int64) (*Block, error) {
	panic("block service client is disabled")
}

// GetLatestBlock implements BlockServiceClient.
func (*disabledBlockServiceClient) GetLatestBlock(context.Context) (*Block, error) {
	panic("block service client is disabled")
}

// GetLatestHeight implements BlockServiceClient GetLatestHeight - disabled client.
func (*disabledBlockServiceClient) GetLatestHeight(context.Context, ...GetLatestHeightOption) (<-chan LatestHeightResult, error) {
	panic("block service client is disabled")
}
