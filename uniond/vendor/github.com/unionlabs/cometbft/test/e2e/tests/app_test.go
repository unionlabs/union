package e2e_test

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"math/rand"
	"strconv"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	cmtbytes "github.com/cometbft/cometbft/libs/bytes"
	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/types"
)

// Tests that any initial state given in genesis has made it into the app.
func TestApp_InitialState(t *testing.T) {
	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		if len(node.Testnet.InitialState) == 0 {
			return
		}

		client, err := node.Client()
		require.NoError(t, err)
		for k, v := range node.Testnet.InitialState {
			resp, err := client.ABCIQuery(ctx, "", []byte(k))
			require.NoError(t, err)
			assert.Equal(t, k, string(resp.Response.Key))
			assert.Equal(t, v, string(resp.Response.Value))
		}
	})
}

// Tests that the app hash (as reported by the app) matches the last
// block and the node sync status.
func TestApp_Hash(t *testing.T) {
	t.Helper()
	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		client, err := node.Client()
		require.NoError(t, err)

		info, err := client.ABCIInfo(ctx)
		require.NoError(t, err)
		require.NotEmpty(t, info.Response.LastBlockAppHash, "expected app to return app hash")

		// In next-block execution, the app hash is stored in the next block
		requestedHeight := info.Response.LastBlockHeight + 1

		require.Eventually(t, func() bool {
			status, err := client.Status(ctx)
			require.NoError(t, err)
			require.NotZero(t, status.SyncInfo.LatestBlockHeight)
			return status.SyncInfo.LatestBlockHeight >= requestedHeight
		}, 30*time.Second, 500*time.Millisecond)

		block, err := client.Block(ctx, &requestedHeight)
		require.NoError(t, err)
		require.Equal(t,
			hex.EncodeToString(info.Response.LastBlockAppHash),
			hex.EncodeToString(block.Block.AppHash.Bytes()),
			"app hash does not match last block's app hash")
	})
}

// Tests that we can set a value and retrieve it.
func TestApp_Tx(t *testing.T) {
	t.Helper()
	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		client, err := node.Client()
		require.NoError(t, err)

		// Generate a random value, to prevent duplicate tx errors when
		// manually running the test multiple times for a testnet.
		r := rand.New(rand.NewSource(time.Now().UnixNano()))
		bz := make([]byte, 32)
		_, err = r.Read(bz)
		require.NoError(t, err)

		key := fmt.Sprintf("testapp-tx-%v", node.Name)
		value := hex.EncodeToString(bz)
		tx := types.Tx(fmt.Sprintf("%v=%v", key, value))

		res, err := client.BroadcastTxSync(ctx, tx)
		require.NoError(t, err)
		require.NotNil(t, res)
		require.Zero(t, res.Code)

		hash := tx.Hash()
		require.Equal(t, res.Hash, cmtbytes.HexBytes(hash))
		waitTime := 30 * time.Second
		require.Eventuallyf(t, func() bool {
			txResp, err := client.Tx(ctx, hash, false)
			return err == nil && bytes.Equal(txResp.Tx, tx)
		}, waitTime, time.Second,
			"submitted tx (%X) wasn't committed after %v", hash, waitTime,
		)

		// NOTE: we don't test abci query of the light client
		if node.Mode == e2e.ModeLight {
			return
		}

		abciResp, err := client.ABCIQuery(ctx, "", []byte(key))
		require.NoError(t, err)
		assert.Equal(t, key, string(abciResp.Response.Key))
		assert.Equal(t, value, string(abciResp.Response.Value))
	})
}

func TestApp_VoteExtensions(t *testing.T) {
	t.Helper()
	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		client, err := node.Client()
		require.NoError(t, err)
		info, err := client.ABCIInfo(ctx)
		require.NoError(t, err)

		// This special value should have been created by way of vote extensions
		resp, err := client.ABCIQuery(ctx, "", []byte("extensionSum"))
		require.NoError(t, err)

		// if extensions are not enabled on the network, we should expect
		// the app to have any extension value set (via a normal tx).
		if node.Testnet.VoteExtensionsEnableHeight != 0 &&
			info.Response.LastBlockHeight > node.Testnet.VoteExtensionsEnableHeight {
			parts := bytes.Split(resp.Response.Value, []byte("|"))
			require.Len(t, parts, 2)
			extSum, err := strconv.Atoi(string(parts[0]))
			require.NoError(t, err)
			require.GreaterOrEqual(t, extSum, 0)
		}
	})
}
