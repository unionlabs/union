package cmtservice

import (
	"context"

	rpcclient "github.com/cometbft/cometbft/rpc/client"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
)

// CometRPC defines the interface of a CometBFT RPC client needed for
// queries and transaction handling.
type CometRPC interface {
	rpcclient.ABCIClient

	Validators(ctx context.Context, height *int64, page, perPage *int) (*coretypes.ResultValidators, error)
	Status(context.Context) (*coretypes.ResultStatus, error)
	Block(ctx context.Context, height *int64) (*coretypes.ResultBlock, error)
	BlockByHash(ctx context.Context, hash []byte) (*coretypes.ResultBlock, error)
	BlockResults(ctx context.Context, height *int64) (*coretypes.ResultBlockResults, error)
	BlockchainInfo(ctx context.Context, minHeight, maxHeight int64) (*coretypes.ResultBlockchainInfo, error)
	Commit(ctx context.Context, height *int64) (*coretypes.ResultCommit, error)
	Tx(ctx context.Context, hash []byte, prove bool) (*coretypes.ResultTx, error)
	TxSearch(
		ctx context.Context,
		query string,
		prove bool,
		page, perPage *int,
		orderBy string,
	) (*coretypes.ResultTxSearch, error)
	BlockSearch(
		ctx context.Context,
		query string,
		page, perPage *int,
		orderBy string,
	) (*coretypes.ResultBlockSearch, error)
}
