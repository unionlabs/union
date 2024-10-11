package cmtservice_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"

	"github.com/cosmos/cosmos-sdk/server"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	"github.com/cosmos/cosmos-sdk/testutil/network"
)

func TestStatusCommand(t *testing.T) {
	t.Skip() // Rewrite as system test

	cfg, err := network.DefaultConfigWithAppConfig(depinject.Configs() /* TODO, test skipped anyway */)
	require.NoError(t, err)

	network, err := network.New(t, t.TempDir(), cfg)
	require.NoError(t, err)
	require.NoError(t, network.WaitForNextBlock())

	val0 := network.GetValidators()[0]
	cmd := server.StatusCommand()

	out, err := clitestutil.ExecTestCLICmd(val0.GetClientCtx(), cmd, []string{})
	require.NoError(t, err)

	// Make sure the output has the validator moniker.
	require.Contains(t, out.String(), fmt.Sprintf("\"moniker\":\"%s\"", val0.GetMoniker()))
}
