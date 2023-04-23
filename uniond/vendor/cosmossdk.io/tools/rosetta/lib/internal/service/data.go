package service

import (
	"context"

	"cosmossdk.io/tools/rosetta/lib/errors"
	crgtypes "cosmossdk.io/tools/rosetta/lib/types"
	"github.com/coinbase/rosetta-sdk-go/types"
)

// AccountBalance retrieves the account balance of an address
// rosetta requires us to fetch the block information too
func (on OnlineNetwork) AccountBalance(ctx context.Context, request *types.AccountBalanceRequest) (*types.AccountBalanceResponse, *types.Error) {
	var (
		height int64
		block  crgtypes.BlockResponse
		err    error
	)

	switch {
	case request.BlockIdentifier == nil:
		syncStatus, err := on.client.Status(ctx)
		if err != nil {
			return nil, errors.ToRosetta(err)
		}
		block, err = on.client.BlockByHeight(ctx, syncStatus.CurrentIndex)
		if err != nil {
			return nil, errors.ToRosetta(err)
		}
	case request.BlockIdentifier.Hash != nil:
		block, err = on.client.BlockByHash(ctx, *request.BlockIdentifier.Hash)
		if err != nil {
			return nil, errors.ToRosetta(err)
		}
		height = block.Block.Index
	case request.BlockIdentifier.Index != nil:
		height = *request.BlockIdentifier.Index
		block, err = on.client.BlockByHeight(ctx, &height)
		if err != nil {
			return nil, errors.ToRosetta(err)
		}
	}

	accountCoins, err := on.client.Balances(ctx, request.AccountIdentifier.Address, &height)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	return &types.AccountBalanceResponse{
		BlockIdentifier: block.Block,
		Balances:        accountCoins,
		Metadata:        nil,
	}, nil
}

// Block gets the transactions in the given block
func (on OnlineNetwork) Block(ctx context.Context, request *types.BlockRequest) (*types.BlockResponse, *types.Error) {
	var (
		blockResponse crgtypes.BlockTransactionsResponse
		err           error
	)

	// When fetching data by BlockIdentifier, it may be possible to only specify the index or hash.
	// If neither property is specified, it is assumed that the client is making a request at the current block.
	switch {
	case request.BlockIdentifier == nil: // unlike AccountBalance(), BlockIdentifer is mandatory by spec 1.4.10.
		err := errors.WrapError(errors.ErrBadArgument, "block identifier needs to be specified")
		return nil, errors.ToRosetta(err)

	case request.BlockIdentifier.Hash != nil:
		blockResponse, err = on.client.BlockTransactionsByHash(ctx, *request.BlockIdentifier.Hash)
		if err != nil {
			return nil, errors.ToRosetta(err)
		}
	case request.BlockIdentifier.Index != nil:
		blockResponse, err = on.client.BlockTransactionsByHeight(ctx, request.BlockIdentifier.Index)
		if err != nil {
			return nil, errors.ToRosetta(err)
		}

	default:
		// both empty
		blockResponse, err = on.client.BlockTransactionsByHeight(ctx, nil)
		if err != nil {
			return nil, errors.ToRosetta(err)
		}
	}

	// Both of index and hash can be specified in reuqest, so make sure they are not mismatching.
	if request.BlockIdentifier.Index != nil && *request.BlockIdentifier.Index != blockResponse.Block.Index {
		err := errors.WrapError(errors.ErrBadArgument, "mismatching index")
		return nil, errors.ToRosetta(err)
	}

	if request.BlockIdentifier.Hash != nil && *request.BlockIdentifier.Hash != blockResponse.Block.Hash {
		err := errors.WrapError(errors.ErrBadArgument, "mismatching hash")
		return nil, errors.ToRosetta(err)
	}

	return &types.BlockResponse{
		Block: &types.Block{
			BlockIdentifier:       blockResponse.Block,
			ParentBlockIdentifier: blockResponse.ParentBlock,
			Timestamp:             blockResponse.MillisecondTimestamp,
			Transactions:          blockResponse.Transactions,
			Metadata:              nil,
		},
		OtherTransactions: nil,
	}, nil
}

// BlockTransaction gets the given transaction in the specified block, we do not need to check the block itself too
// due to the fact that tendermint achieves instant finality
func (on OnlineNetwork) BlockTransaction(ctx context.Context, request *types.BlockTransactionRequest) (*types.BlockTransactionResponse, *types.Error) {
	tx, err := on.client.GetTx(ctx, request.TransactionIdentifier.Hash)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	return &types.BlockTransactionResponse{
		Transaction: tx,
	}, nil
}

// Mempool fetches the transactions contained in the mempool
func (on OnlineNetwork) Mempool(ctx context.Context, _ *types.NetworkRequest) (*types.MempoolResponse, *types.Error) {
	txs, err := on.client.Mempool(ctx)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	return &types.MempoolResponse{
		TransactionIdentifiers: txs,
	}, nil
}

// MempoolTransaction fetches a single transaction in the mempool
// NOTE: it is not implemented yet
func (on OnlineNetwork) MempoolTransaction(ctx context.Context, request *types.MempoolTransactionRequest) (*types.MempoolTransactionResponse, *types.Error) {
	tx, err := on.client.GetUnconfirmedTx(ctx, request.TransactionIdentifier.Hash)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	return &types.MempoolTransactionResponse{
		Transaction: tx,
	}, nil
}

func (on OnlineNetwork) NetworkList(_ context.Context, _ *types.MetadataRequest) (*types.NetworkListResponse, *types.Error) {
	return &types.NetworkListResponse{NetworkIdentifiers: []*types.NetworkIdentifier{on.network}}, nil
}

func (on OnlineNetwork) NetworkOptions(_ context.Context, _ *types.NetworkRequest) (*types.NetworkOptionsResponse, *types.Error) {
	return on.networkOptions, nil
}

func (on OnlineNetwork) NetworkStatus(ctx context.Context, _ *types.NetworkRequest) (*types.NetworkStatusResponse, *types.Error) {
	syncStatus, err := on.client.Status(ctx)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	block, err := on.client.BlockByHeight(ctx, syncStatus.CurrentIndex)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	peers, err := on.client.Peers(ctx)
	if err != nil {
		return nil, errors.ToRosetta(err)
	}

	return &types.NetworkStatusResponse{
		CurrentBlockIdentifier: block.Block,
		CurrentBlockTimestamp:  block.MillisecondTimestamp,
		GenesisBlockIdentifier: on.genesisBlockIdentifier,
		OldestBlockIdentifier:  nil,
		SyncStatus:             syncStatus,
		Peers:                  peers,
	}, nil
}
