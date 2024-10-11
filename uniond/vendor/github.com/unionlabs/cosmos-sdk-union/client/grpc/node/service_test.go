package node

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/server/config"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestServiceServer_Config(t *testing.T) {
	defaultCfg := config.DefaultConfig()
	defaultCfg.PruningKeepRecent = "2000"
	defaultCfg.PruningInterval = "10"
	defaultCfg.HaltHeight = 100
	svr := NewQueryServer(client.Context{}, *defaultCfg)
	ctx := sdk.Context{}.WithMinGasPrices(sdk.NewDecCoins(sdk.NewInt64DecCoin("stake", 15)))

	resp, err := svr.Config(ctx, &ConfigRequest{})
	require.NoError(t, err)
	require.NotNil(t, resp)
	require.Equal(t, ctx.MinGasPrices().String(), resp.MinimumGasPrice)
	require.Equal(t, defaultCfg.PruningKeepRecent, resp.PruningKeepRecent)
	require.Equal(t, defaultCfg.PruningInterval, resp.PruningInterval)
	require.Equal(t, defaultCfg.HaltHeight, resp.HaltHeight)
}
