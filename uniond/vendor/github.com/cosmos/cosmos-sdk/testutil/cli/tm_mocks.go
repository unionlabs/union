package cli

import (
	"context"

	abci "github.com/cometbft/cometbft/abci/types"
	tmbytes "github.com/cometbft/cometbft/libs/bytes"
	rpcclient "github.com/cometbft/cometbft/rpc/client"
	rpcclientmock "github.com/cometbft/cometbft/rpc/client/mock"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	tmtypes "github.com/cometbft/cometbft/types"

	"github.com/cosmos/cosmos-sdk/client"
)

var _ client.TendermintRPC = (*MockTendermintRPC)(nil)

type MockTendermintRPC struct {
	rpcclientmock.Client

	responseQuery abci.ResponseQuery
}

// NewMockTendermintRPC returns a mock TendermintRPC implementation.
// It is used for CLI testing.
func NewMockTendermintRPC(respQuery abci.ResponseQuery) MockTendermintRPC {
	return MockTendermintRPC{responseQuery: respQuery}
}

func (MockTendermintRPC) BroadcastTxSync(context.Context, tmtypes.Tx) (*coretypes.ResultBroadcastTx, error) {
	return &coretypes.ResultBroadcastTx{Code: 0}, nil
}

func (m MockTendermintRPC) ABCIQueryWithOptions(
	_ context.Context,
	_ string,
	_ tmbytes.HexBytes,
	_ rpcclient.ABCIQueryOptions,
) (*coretypes.ResultABCIQuery, error) {
	return &coretypes.ResultABCIQuery{Response: m.responseQuery}, nil
}
