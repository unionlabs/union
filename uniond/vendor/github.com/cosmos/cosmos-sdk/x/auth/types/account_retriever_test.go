package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil/network"
	"github.com/cosmos/cosmos-sdk/x/auth/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestAccountRetriever(t *testing.T) {
	cfg, err := network.DefaultConfigWithAppConfig(testutil.AppConfig)
	require.NoError(t, err)
	cfg.NumValidators = 1

	network, err := network.New(t, t.TempDir(), cfg)
	require.NoError(t, err)
	defer network.Cleanup()

	_, err = network.WaitForHeight(3)
	require.NoError(t, err)

	val := network.Validators[0]
	clientCtx := val.ClientCtx
	ar := types.AccountRetriever{}

	clientCtx = clientCtx.WithHeight(2)

	acc, err := ar.GetAccount(clientCtx, val.Address)
	require.NoError(t, err)
	require.NotNil(t, acc)

	acc, height, err := ar.GetAccountWithHeight(clientCtx, val.Address)
	require.NoError(t, err)
	require.NotNil(t, acc)
	require.Equal(t, height, int64(2))

	require.NoError(t, ar.EnsureExists(clientCtx, val.Address))

	accNum, accSeq, err := ar.GetAccountNumberSequence(clientCtx, val.Address)
	require.NoError(t, err)
	require.Equal(t, accNum, uint64(0))
	require.Equal(t, accSeq, uint64(1))
}
