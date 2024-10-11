package cmd_test

import (
	"fmt"
	"os"
	"strings"
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/tools/confix/cmd"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/config"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
)

// initClientContext initiates client Context for tests
func initClientContext(t *testing.T) (client.Context, func()) {
	home := t.TempDir()
	chainID := "test-chain"
	clientCtx := client.Context{}.
		WithHomeDir(home).
		WithViper("").
		WithChainID(chainID)
	clientCtx, err := config.ReadFromClientConfig(clientCtx)
	assert.NilError(t, err)
	assert.Equal(t, clientCtx.ChainID, chainID)
	return clientCtx, func() { _ = os.RemoveAll(home) }
}

func TestSetCmd(t *testing.T) {
	clientCtx, cleanup := initClientContext(t)
	defer cleanup()

	_, err := clitestutil.ExecTestCLICmd(clientCtx, cmd.SetCommand(), []string{"unexisting", "foo", "foo"})
	assert.ErrorContains(t, err, "no such file or directory")

	out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd.GetCommand(), []string{"client", "chain-id"})
	assert.NilError(t, err)
	assert.Equal(t, strings.TrimSpace(out.String()), `"test-chain"`)

	newValue := "new-chain"
	_, err = clitestutil.ExecTestCLICmd(clientCtx, cmd.SetCommand(), []string{"client", "chain-id", newValue})
	assert.NilError(t, err)

	out, err = clitestutil.ExecTestCLICmd(clientCtx, cmd.GetCommand(), []string{"client", "chain-id"})
	assert.NilError(t, err)
	assert.Equal(t, strings.TrimSpace(out.String()), fmt.Sprintf(`"%s"`, newValue))
}

func TestGetCmd(t *testing.T) {
	clientCtx, cleanup := initClientContext(t)
	defer cleanup()

	out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd.GetCommand(), []string{"client", "chain-id"})
	assert.NilError(t, err)
	assert.Equal(t, strings.TrimSpace(out.String()), `"test-chain"`)

	_, err = clitestutil.ExecTestCLICmd(clientCtx, cmd.GetCommand(), []string{"client", "foo"})
	assert.Error(t, err, `key "foo" not found`)

	_, err = clitestutil.ExecTestCLICmd(clientCtx, cmd.GetCommand(), []string{"unexisting", "foo"})
	assert.ErrorContains(t, err, "no such file or directory")
}
