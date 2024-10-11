package e2e_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/version"
)

// These tests are in place to confirm that both the non-privileged and privileged GRPC services can be called upon
// successfully and produce the expected outcomes. They consist of straightforward method invocations for each service.
// The emphasis is on avoiding complex scenarios and excluding hard-to-test cases like pruning logic.

// Test the GRPC Version service. Invoke the GetVersion method.
func TestGRPC_Version(t *testing.T) {
	t.Helper()
	testFullNodesOrValidators(t, 0, func(t *testing.T, node e2e.Node) {
		t.Helper()
		ctx, ctxCancel := context.WithTimeout(context.Background(), time.Minute)
		defer ctxCancel()
		client, err := node.GRPCClient(ctx)
		require.NoError(t, err)
		defer client.Close()

		res, err := client.GetVersion(ctx)
		require.NoError(t, err)

		require.Equal(t, version.CMTSemVer, res.Node)
		require.Equal(t, version.ABCIVersion, res.ABCI)
		require.Equal(t, version.P2PProtocol, res.P2P)
		require.Equal(t, version.BlockProtocol, res.Block)
	})
}

// Test the GRPC Block Service. Invoke the GetBlockByHeight method to return a block
// at the latest height returned by the Block Service's GetLatestHeight method.
func TestGRPC_Block_GetByHeight(t *testing.T) {
	testFullNodesOrValidators(t, 0, func(t *testing.T, node e2e.Node) {
		t.Helper()

		// Get the latest height
		latestHeight, err := getLatestHeight(node)
		require.NoError(t, err)

		ctx, ctxCancel := context.WithTimeout(context.Background(), time.Minute)
		defer ctxCancel()

		gRPCClient, err := node.GRPCClient(ctx)
		require.NoError(t, err)
		defer gRPCClient.Close()

		// Get last block and fetch it using the gRPC endpoint
		lastBlock, err := gRPCClient.GetBlockByHeight(ctx, latestHeight)

		// Last block tests. Check if heights match, the latest height retrieved and the block height fetched
		require.NoError(t, err)
		require.NotNil(t, lastBlock.BlockID)
		require.Equal(t, lastBlock.Block.Height, latestHeight)
		require.NotNil(t, lastBlock.Block.LastCommit)
	})
}

// Test the GRPC Block Results service. Invoke the GetBlockResults method to retrieve the block results
// at the latest height returned by the Block Service's GetLatestHeight method.
func TestGRPC_GetBlockResults(t *testing.T) {
	t.Helper()
	testFullNodesOrValidators(t, 0, func(t *testing.T, node e2e.Node) {
		t.Helper()

		// Get the latest height
		latestHeight, err := getLatestHeight(node)
		require.NoError(t, err)

		ctx, ctxCancel := context.WithTimeout(context.Background(), time.Minute)
		defer ctxCancel()

		gRPCClient, err := node.GRPCClient(ctx)
		require.NoError(t, err)
		defer gRPCClient.Close()

		// Fetch the block results at the latest height retrieved
		// Ensure the heights match, the latest height used to fetch the Block Results and
		// the height returned in the block results.
		// Also ensure the AppHash is not nil.
		blockResults, err := gRPCClient.GetBlockResults(ctx, latestHeight)
		require.NoError(t, err)
		require.Equal(t, blockResults.Height, latestHeight)
		require.NotNil(t, blockResults.AppHash)
	})
}

// Test the GRPC Privileged Pruning Service methods to set and get the block retain height.
func TestGRPC_BlockRetainHeight(t *testing.T) {
	t.Helper()
	testFullNodesOrValidators(t, 0, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if !node.EnableCompanionPruning {
			return
		}

		// Get the latest height
		latestHeight, err := getLatestHeight(node)
		require.NoError(t, err)

		ctx, ctxCancel := context.WithTimeout(context.Background(), time.Minute)
		defer ctxCancel()

		gRPCClient, err := node.GRPCPrivilegedClient(ctx)
		require.NoError(t, err)
		defer gRPCClient.Close()

		// Test the setting the block retain height method from the GRPC Pruning service
		// Ensure that the height set matches the retrieved retain height
		err = gRPCClient.SetBlockRetainHeight(ctx, uint64(latestHeight-1))
		require.NoError(t, err)
		res, err := gRPCClient.GetBlockRetainHeight(ctx)
		require.NoError(t, err)
		require.NotNil(t, res)
		require.Equal(t, res.PruningService, uint64(latestHeight-1))
	})
}

// Test the GRPC Privileged Pruning Service methods to set and get the block results retain height.
func TestGRPC_BlockResultsRetainHeight(t *testing.T) {
	t.Helper()
	testFullNodesOrValidators(t, 0, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if !node.EnableCompanionPruning {
			return
		}

		// Get the latest height
		latestHeight, err := getLatestHeight(node)
		require.NoError(t, err)

		ctx, ctxCancel := context.WithTimeout(context.Background(), time.Minute)
		defer ctxCancel()

		gRPCClient, err := node.GRPCPrivilegedClient(ctx)
		require.NoError(t, err)
		defer gRPCClient.Close()

		// Test the setting the block results retain height method from the GRPC Pruning service
		// Ensure that the height set matches the retrieved retain height
		err = gRPCClient.SetBlockResultsRetainHeight(ctx, uint64(latestHeight-1))
		require.NoError(t, err)
		height, err := gRPCClient.GetBlockResultsRetainHeight(ctx)
		require.NoError(t, err)
		require.Equal(t, height, uint64(latestHeight-1))
	})
}

// Test the GRPC Privileged Pruning Service methods to set and get the tx indexer retain height.
func TestGRPC_TxIndexerRetainHeight(t *testing.T) {
	testFullNodesOrValidators(t, 0, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if !node.EnableCompanionPruning {
			return
		}

		// Get the latest height
		latestHeight, err := getLatestHeight(node)
		require.NoError(t, err)

		ctx, ctxCancel := context.WithTimeout(context.Background(), time.Minute)
		defer ctxCancel()

		gRPCClient, err := node.GRPCPrivilegedClient(ctx)
		require.NoError(t, err)
		defer gRPCClient.Close()

		// Test the setting the tx indexer retain height method from the GRPC Pruning service
		// Ensure that the height set matches the retrieved retain height
		err = gRPCClient.SetTxIndexerRetainHeight(ctx, uint64(latestHeight-1))
		require.NoError(t, err)
		height, err := gRPCClient.GetTxIndexerRetainHeight(ctx)
		require.NoError(t, err)
		require.Equal(t, height, uint64(latestHeight-1))
	})
}

// Test the GRPC Privileged Pruning Service methods to set and get the block indexer retain height.
func TestGRPC_BlockIndexerRetainHeight(t *testing.T) {
	t.Helper()
	testFullNodesOrValidators(t, 0, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if !node.EnableCompanionPruning {
			return
		}

		// Get the latest height
		latestHeight, err := getLatestHeight(node)
		require.NoError(t, err)

		ctx, ctxCancel := context.WithTimeout(context.Background(), time.Minute)
		defer ctxCancel()

		gRPCClient, err := node.GRPCPrivilegedClient(ctx)
		require.NoError(t, err)
		defer gRPCClient.Close()

		// Test the setting the block indexer retain height method from the GRPC Pruning service
		// Ensure that the height set matches the retrieved retain height
		err = gRPCClient.SetBlockIndexerRetainHeight(ctx, uint64(latestHeight-1))
		require.NoError(t, err)
		height, err := gRPCClient.GetBlockIndexerRetainHeight(ctx)
		require.NoError(t, err)
		require.Equal(t, height, uint64(latestHeight-1))
	})
}

// This method returns the latest height retrieved from the GRPC Block Service invoking the
// GetLatestHeight, which returns a channel that receives the latest height. Once a height is
// received in the channel, return that height.
func getLatestHeight(node e2e.Node) (int64, error) {
	ctx, ctxCancel := context.WithTimeout(context.Background(), 3*time.Minute) // 3 minute timeout
	defer ctxCancel()
	gRPCClient, err := node.GRPCClient(ctx)
	if err != nil {
		return 0, err
	}
	defer gRPCClient.Close()
	latestHeightCh, err := gRPCClient.GetLatestHeight(ctx)
	if err != nil {
		return 0, err
	}

	for {
		select {
		case <-ctx.Done():
			if errors.Is(ctx.Err(), context.DeadlineExceeded) {
				// Context has timed out
				return 0, errors.New("context deadline exceeded while waiting for latest height")
			}
			// Context has been canceled
			return 0, ctx.Err()
		case latest, ok := <-latestHeightCh:
			if ok {
				return latest.Height, nil
			} else {
				return 0, errors.New("failed to receive latest height from channel")
			}
		}
	}
}
