package rpc_test

import (
	"context"
	"strconv"
	"testing"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"

	"github.com/cosmos/cosmos-sdk/client"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/types/address"
	grpctypes "github.com/cosmos/cosmos-sdk/types/grpc"
)

func TestCLIQueryConn(t *testing.T) {
	t.Skip("data race in comet is causing this to fail")

	var header metadata.MD

	testClient := testdata.NewQueryClient(client.Context{})
	res, err := testClient.Echo(context.Background(), &testdata.EchoRequest{Message: "hello"}, grpc.Header(&header))
	require.NoError(t, err)

	blockHeight := header.Get(grpctypes.GRPCBlockHeightHeader)
	height, err := strconv.Atoi(blockHeight[0])
	require.NoError(t, err)
	require.GreaterOrEqual(t, height, 1) // at least the 1st block
	require.Equal(t, "hello", res.Message)
}

func TestQueryABCIHeight(t *testing.T) {
	testCases := []struct {
		name      string
		reqHeight int64
		ctxHeight int64
		expHeight int64
	}{
		{
			name:      "non zero request height",
			reqHeight: 3,
			ctxHeight: 1, // query at height 1 or 2 would cause an error
			expHeight: 3,
		},
		{
			name:      "empty request height - use context height",
			reqHeight: 0,
			ctxHeight: 3,
			expHeight: 3,
		},
		{
			name:      "empty request height and context height - use latest height",
			reqHeight: 0,
			ctxHeight: 0,
			expHeight: 4,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			req := abci.QueryRequest{
				Path:   "store/bank/key",
				Height: tc.reqHeight,
				Data:   address.MustLengthPrefix([]byte{}),
				Prove:  true,
			}

			clientCtx := client.Context{}.WithHeight(tc.ctxHeight).
				WithClient(clitestutil.NewMockCometRPC(abci.QueryResponse{
					Height: tc.expHeight,
				}))

			res, err := clientCtx.QueryABCI(req)
			require.NoError(t, err)
			require.Equal(t, tc.expHeight, res.Height)
		})
	}
}
