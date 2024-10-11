package http_test

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/abci/example/kvstore"
	"github.com/cometbft/cometbft/light/provider"
	lighthttp "github.com/cometbft/cometbft/light/provider/http"
	rpcclient "github.com/cometbft/cometbft/rpc/client"
	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	rpctest "github.com/cometbft/cometbft/rpc/test"
	"github.com/cometbft/cometbft/types"
)

func TestNewProvider(t *testing.T) {
	c, err := lighthttp.New("chain-test", "192.168.0.1:26657")
	require.NoError(t, err)
	require.Equal(t, "http{http://192.168.0.1:26657}", fmt.Sprintf("%s", c))

	c, err = lighthttp.New("chain-test", "http://153.200.0.1:26657")
	require.NoError(t, err)
	require.Equal(t, "http{http://153.200.0.1:26657}", fmt.Sprintf("%s", c))

	c, err = lighthttp.New("chain-test", "153.200.0.1")
	require.NoError(t, err)
	require.Equal(t, "http{http://153.200.0.1}", fmt.Sprintf("%s", c))
}

func TestProvider(t *testing.T) {
	for _, path := range []string{"", "/", "/v1", "/v1/"} {
		app := kvstore.NewInMemoryApplication()
		app.RetainBlocks = 10
		node := rpctest.StartCometBFT(app, rpctest.RecreateConfig)

		cfg := rpctest.GetConfig()
		defer os.RemoveAll(cfg.RootDir)
		rpcAddr := cfg.RPC.ListenAddress
		genDoc, err := types.GenesisDocFromFile(cfg.GenesisFile())
		require.NoError(t, err)
		chainID := genDoc.ChainID

		c, err := rpchttp.New(rpcAddr + path)
		require.NoError(t, err)

		p := lighthttp.NewWithClient(chainID, c)
		require.NoError(t, err)
		require.NotNil(t, p)

		// let it produce some blocks
		err = rpcclient.WaitForHeight(c, 10, nil)
		require.NoError(t, err)

		// let's get the highest block
		lb, err := p.LightBlock(context.Background(), 0)
		require.NoError(t, err)
		require.NotNil(t, lb)
		assert.GreaterOrEqual(t, lb.Height, int64(10))

		// let's check this is valid somehow
		require.NoError(t, lb.ValidateBasic(chainID))

		// historical queries now work :)
		lb, err = p.LightBlock(context.Background(), 0)
		require.NoError(t, err)
		require.NotNil(t, lb)
		lower := lb.Height - 3
		lb, err = p.LightBlock(context.Background(), lower)
		require.NoError(t, err)
		assert.Equal(t, lower, lb.Height)

		// fetching missing heights (both future and pruned) should return appropriate errors
		lb, err = p.LightBlock(context.Background(), 0)
		require.NoError(t, err)
		require.NotNil(t, lb)
		lb, err = p.LightBlock(context.Background(), lb.Height+100000)
		require.Error(t, err)
		require.Nil(t, lb)
		assert.Equal(t, provider.ErrHeightTooHigh, err)

		_, err = p.LightBlock(context.Background(), 1)
		require.Error(t, err)
		require.Nil(t, lb)
		assert.Equal(t, provider.ErrLightBlockNotFound, err)

		// stop the full node and check that a no response error is returned
		rpctest.StopCometBFT(node)
		time.Sleep(10 * time.Second)
		lb, err = p.LightBlock(context.Background(), lower+2)
		// we should see a connection refused
		require.Error(t, err)
		require.Contains(t, err.Error(), "connection refused")
		require.Nil(t, lb)
	}
}
