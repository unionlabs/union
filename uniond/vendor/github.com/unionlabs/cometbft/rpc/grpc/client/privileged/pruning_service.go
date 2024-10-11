package privileged

import (
	"context"

	"github.com/cosmos/gogoproto/grpc"

	pbsvc "github.com/cometbft/cometbft/api/cometbft/services/pruning/v1"
)

// RetainHeights provides information on which block height limits have been
// set for block information to be retained by the ABCI application and the
// pruning service.
type RetainHeights struct {
	App            uint64
	PruningService uint64
}

type PruningServiceClient interface {
	SetBlockRetainHeight(ctx context.Context, height uint64) error
	GetBlockRetainHeight(ctx context.Context) (RetainHeights, error)
	SetBlockResultsRetainHeight(ctx context.Context, height uint64) error
	GetBlockResultsRetainHeight(ctx context.Context) (uint64, error)
	SetTxIndexerRetainHeight(ctx context.Context, height uint64) error
	GetTxIndexerRetainHeight(ctx context.Context) (uint64, error)
	SetBlockIndexerRetainHeight(ctx context.Context, height uint64) error
	GetBlockIndexerRetainHeight(ctx context.Context) (uint64, error)
}

type pruningServiceClient struct {
	inner pbsvc.PruningServiceClient
}

func newPruningServiceClient(conn grpc.ClientConn) PruningServiceClient {
	return &pruningServiceClient{
		inner: pbsvc.NewPruningServiceClient(conn),
	}
}

func (c *pruningServiceClient) SetBlockIndexerRetainHeight(ctx context.Context, height uint64) error {
	_, err := c.inner.SetBlockIndexerRetainHeight(ctx, &pbsvc.SetBlockIndexerRetainHeightRequest{
		Height: height,
	})
	return err
}

func (c *pruningServiceClient) GetBlockIndexerRetainHeight(ctx context.Context) (uint64, error) {
	res, err := c.inner.GetBlockIndexerRetainHeight(ctx, &pbsvc.GetBlockIndexerRetainHeightRequest{})
	if err != nil {
		return 0, err
	}
	return res.Height, nil
}

// SetTxIndexerRetainHeight implements PruningServiceClient.
func (c *pruningServiceClient) SetTxIndexerRetainHeight(ctx context.Context, height uint64) error {
	_, err := c.inner.SetTxIndexerRetainHeight(ctx, &pbsvc.SetTxIndexerRetainHeightRequest{
		Height: height,
	})
	return err
}

// GetTxIndexerRetainHeight implements PruningServiceClient.
func (c *pruningServiceClient) GetTxIndexerRetainHeight(ctx context.Context) (uint64, error) {
	res, err := c.inner.GetTxIndexerRetainHeight(ctx, &pbsvc.GetTxIndexerRetainHeightRequest{})
	if err != nil {
		return 0, err
	}
	return res.Height, nil
}

// SetBlockRetainHeight implements PruningServiceClient.
func (c *pruningServiceClient) SetBlockRetainHeight(ctx context.Context, height uint64) error {
	_, err := c.inner.SetBlockRetainHeight(ctx, &pbsvc.SetBlockRetainHeightRequest{
		Height: height,
	})
	return err
}

// GetBlockRetainHeight implements PruningServiceClient.
func (c *pruningServiceClient) GetBlockRetainHeight(ctx context.Context) (RetainHeights, error) {
	res, err := c.inner.GetBlockRetainHeight(ctx, &pbsvc.GetBlockRetainHeightRequest{})
	if err != nil {
		return RetainHeights{}, err
	}
	return RetainHeights{
		App:            res.AppRetainHeight,
		PruningService: res.PruningServiceRetainHeight,
	}, nil
}

// SetBlockResultsRetainHeight implements PruningServiceClient.
func (c *pruningServiceClient) SetBlockResultsRetainHeight(ctx context.Context, height uint64) error {
	_, err := c.inner.SetBlockResultsRetainHeight(ctx, &pbsvc.SetBlockResultsRetainHeightRequest{
		Height: height,
	})
	return err
}

// GetBlockResultsRetainHeight implements PruningServiceClient.
func (c *pruningServiceClient) GetBlockResultsRetainHeight(ctx context.Context) (uint64, error) {
	res, err := c.inner.GetBlockResultsRetainHeight(ctx, &pbsvc.GetBlockResultsRetainHeightRequest{})
	if err != nil {
		return 0, err
	}
	return res.PruningServiceRetainHeight, nil
}

type disabledPruningServiceClient struct{}

func newDisabledPruningServiceClient() PruningServiceClient {
	return &disabledPruningServiceClient{}
}

// SetBlockRetainHeight implements PruningServiceClient.
func (*disabledPruningServiceClient) SetBlockRetainHeight(context.Context, uint64) error {
	panic("pruning service client is disabled")
}

// GetBlockRetainHeight implements PruningServiceClient.
func (*disabledPruningServiceClient) GetBlockRetainHeight(context.Context) (RetainHeights, error) {
	panic("pruning service client is disabled")
}

// SetBlockResultsRetainHeight implements PruningServiceClient.
func (*disabledPruningServiceClient) SetBlockResultsRetainHeight(context.Context, uint64) error {
	panic("pruning service client is disabled")
}

// GetBlockResultsRetainHeight implements PruningServiceClient.
func (*disabledPruningServiceClient) GetBlockResultsRetainHeight(context.Context) (uint64, error) {
	panic("pruning service client is disabled")
}

func (*disabledPruningServiceClient) SetTxIndexerRetainHeight(context.Context, uint64) error {
	panic("pruning service client is disabled")
}

func (*disabledPruningServiceClient) GetTxIndexerRetainHeight(context.Context) (uint64, error) {
	panic("pruning service client is disabled")
}

func (*disabledPruningServiceClient) SetBlockIndexerRetainHeight(context.Context, uint64) error {
	panic("pruning service client is disabled")
}

func (*disabledPruningServiceClient) GetBlockIndexerRetainHeight(context.Context) (uint64, error) {
	panic("pruning service client is disabled")
}
