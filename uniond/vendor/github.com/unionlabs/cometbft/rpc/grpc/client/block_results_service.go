package client

import (
	"context"
	"fmt"

	"github.com/cosmos/gogoproto/grpc"

	abci "github.com/cometbft/cometbft/abci/types"
	brs "github.com/cometbft/cometbft/api/cometbft/services/block_results/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
)

type BlockResults struct {
	Height                int64                     `json:"height"`
	TxResults             []*abci.ExecTxResult      `json:"txs_results"`
	FinalizeBlockEvents   []*abci.Event             `json:"finalize_block_events"`
	ValidatorUpdates      []*abci.ValidatorUpdate   `json:"validator_updates"`
	ConsensusParamUpdates *cmtproto.ConsensusParams `json:"consensus_param_updates"`
	AppHash               []byte                    `json:"app_hash"`
}

// BlockResultsServiceClient provides the block results of a given height (or latest if none provided).
type BlockResultsServiceClient interface {
	GetBlockResults(ctx context.Context, height int64) (*BlockResults, error)
}

type blockResultServiceClient struct {
	client brs.BlockResultsServiceClient
}

func (b blockResultServiceClient) GetBlockResults(ctx context.Context, height int64) (*BlockResults, error) {
	res, err := b.client.GetBlockResults(ctx, &brs.GetBlockResultsRequest{Height: height})
	if err != nil {
		return nil, fmt.Errorf("error fetching BlockResults for height %d:: %s", height, err.Error())
	}

	return &BlockResults{
		Height:                res.Height,
		TxResults:             res.TxResults,
		FinalizeBlockEvents:   res.FinalizeBlockEvents,
		ValidatorUpdates:      res.ValidatorUpdates,
		ConsensusParamUpdates: res.ConsensusParamUpdates,
		AppHash:               res.AppHash,
	}, nil
}

func newBlockResultsServiceClient(conn grpc.ClientConn) BlockResultsServiceClient {
	return &blockResultServiceClient{
		client: brs.NewBlockResultsServiceClient(conn),
	}
}

type disabledBlockResultsServiceClient struct{}

func newDisabledBlockResultsServiceClient() BlockResultsServiceClient {
	return &disabledBlockResultsServiceClient{}
}

// GetBlockResults implements BlockResultsServiceClient.
func (*disabledBlockResultsServiceClient) GetBlockResults(_ context.Context, _ int64) (*BlockResults, error) {
	panic("block results service client is disabled")
}
