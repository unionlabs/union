package cmd_test

import (
	"fmt"
	"os"
	"strings"
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/tools/confix/cmd"

	"github.com/cosmos/cosmos-sdk/client"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
)

func TestViewCmd(t *testing.T) {
	clientCtx, cleanup := initClientContext(t)
	defer cleanup()

	_, err := clitestutil.ExecTestCLICmd(clientCtx, cmd.ViewCommand(), []string{"unexisting"})
	assert.ErrorContains(t, err, "no such file or directory")

	expectedCfg := fmt.Sprintf("%s/config/client.toml", clientCtx.HomeDir)
	bz, err := os.ReadFile(expectedCfg)
	assert.NilError(t, err)

	out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd.ViewCommand(), []string{"client"})
	assert.NilError(t, err)
	assert.DeepEqual(t, strings.TrimSpace(out.String()), strings.TrimSpace(string(bz)))

	out, err = clitestutil.ExecTestCLICmd(client.Context{}, cmd.ViewCommand(), []string{fmt.Sprintf("%s/config/client.toml", clientCtx.HomeDir)})
	assert.NilError(t, err)
	assert.DeepEqual(t, strings.TrimSpace(out.String()), strings.TrimSpace(string(bz)))

	out, err = clitestutil.ExecTestCLICmd(clientCtx, cmd.ViewCommand(), []string{"client", "--output-format", "json"})
	assert.NilError(t, err)
	assert.Assert(t, strings.Contains(out.String(), `"chain-id": "test-chain"`))
}
