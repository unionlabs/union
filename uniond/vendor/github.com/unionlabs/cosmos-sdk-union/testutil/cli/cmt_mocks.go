package cli

import (
	"context"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtbytes "github.com/cometbft/cometbft/libs/bytes"
	rpcclient "github.com/cometbft/cometbft/rpc/client"
	rpcclientmock "github.com/cometbft/cometbft/rpc/client/mock"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	cmttypes "github.com/cometbft/cometbft/types"

	"github.com/cosmos/cosmos-sdk/client"
)

var _ client.CometRPC = (*MockCometRPC)(nil)

type MockCometRPC struct {
	rpcclientmock.Client

	responseQuery abci.QueryResponse
}

// NewMockCometRPC returns a mock CometBFT RPC implementation.
// It is used for CLI testing.
func NewMockCometRPC(respQuery abci.QueryResponse) MockCometRPC {
	return MockCometRPC{responseQuery: respQuery}
}

// NewMockCometRPCWithResponseQueryValue returns a mock CometBFT RPC implementation with value only.
// It is used for CLI testing.
func NewMockCometRPCWithResponseQueryValue(bz []byte) MockCometRPC {
	return MockCometRPC{responseQuery: abci.QueryResponse{
		Value: bz,
	}}
}

func (MockCometRPC) BroadcastTxSync(context.Context, cmttypes.Tx) (*coretypes.ResultBroadcastTx, error) {
	return &coretypes.ResultBroadcastTx{Code: 0}, nil
}

func (m MockCometRPC) ABCIQueryWithOptions(
	_ context.Context,
	_ string,
	_ cmtbytes.HexBytes,
	_ rpcclient.ABCIQueryOptions,
) (*coretypes.ResultABCIQuery, error) {
	return &coretypes.ResultABCIQuery{Response: m.responseQuery}, nil
}

type FilterTxsFn = func(query string, start, end int) ([][]byte, error)

type MockCometTxSearchRPC struct {
	rpcclientmock.Client

	txConfig    client.TxConfig
	txs         []cmttypes.Tx
	filterTxsFn FilterTxsFn
}

func (m MockCometTxSearchRPC) Txs() []cmttypes.Tx {
	return m.txs
}

// accept [][]byte so that module that use this for testing dont have to import comet directly
func (m *MockCometTxSearchRPC) WithTxs(txs [][]byte) {
	cmtTxs := make([]cmttypes.Tx, len(txs))
	for i, tx := range txs {
		cmtTxs[i] = tx
	}
	m.txs = cmtTxs
}

func (m *MockCometTxSearchRPC) WithTxConfig(cfg client.TxConfig) {
	m.txConfig = cfg
}

func (m *MockCometTxSearchRPC) WithFilterTxsFn(fn FilterTxsFn) {
	m.filterTxsFn = fn
}

func (MockCometTxSearchRPC) BroadcastTxSync(context.Context, cmttypes.Tx) (*coretypes.ResultBroadcastTx, error) {
	return &coretypes.ResultBroadcastTx{Code: 0}, nil
}

func (mock MockCometTxSearchRPC) TxSearch(ctx context.Context, query string, prove bool, page, perPage *int, orderBy string) (*coretypes.ResultTxSearch, error) {
	if page == nil {
		*page = 0
	}

	if perPage == nil {
		*perPage = 0
	}

	start, end := client.Paginate(len(mock.txs), *page, *perPage, 100)
	if start < 0 || end < 0 {
		// nil result with nil error crashes utils.QueryTxsByEvents
		return &coretypes.ResultTxSearch{}, nil
	}

	var txs []cmttypes.Tx
	if mock.filterTxsFn != nil {
		filterTxs, err := mock.filterTxsFn(query, start, end)
		if err != nil {
			return nil, err
		}

		cmtTxs := make([]cmttypes.Tx, len(filterTxs))
		for i, tx := range filterTxs {
			cmtTxs[i] = tx
		}
		txs = append(txs, cmtTxs...)
	} else {
		txs = mock.txs[start:end]
	}

	rst := &coretypes.ResultTxSearch{Txs: make([]*coretypes.ResultTx, len(txs)), TotalCount: len(txs)}
	for i := range txs {
		rst.Txs[i] = &coretypes.ResultTx{Tx: txs[i]}
	}
	return rst, nil
}

func (mock MockCometTxSearchRPC) Block(ctx context.Context, height *int64) (*coretypes.ResultBlock, error) {
	return &coretypes.ResultBlock{Block: &cmttypes.Block{}}, nil
}
