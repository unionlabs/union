package cmd_test

import (
	"fmt"
	"strings"
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/tools/confix/cmd"

	"github.com/cosmos/cosmos-sdk/client"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
)

func TestMigradeCmd(t *testing.T) {
	clientCtx, cleanup := initClientContext(t)
	defer cleanup()

	_, err := clitestutil.ExecTestCLICmd(client.Context{}, cmd.MigrateCommand(), []string{"v0.0"})
	assert.ErrorContains(t, err, "must provide a path to the app.toml file")

	_, err = clitestutil.ExecTestCLICmd(clientCtx, cmd.MigrateCommand(), []string{"v0.0"})
	assert.ErrorContains(t, err, "unknown version")

	// clientCtx does not create app.toml, so this should fail
	_, err = clitestutil.ExecTestCLICmd(clientCtx, cmd.MigrateCommand(), []string{"v0.45"})
	assert.ErrorContains(t, err, "no such file or directory")

	// try to migrate from client.toml it should fail without --skip-validate
	_, err = clitestutil.ExecTestCLICmd(clientCtx, cmd.MigrateCommand(), []string{"v0.46", fmt.Sprintf("%s/config/client.toml", clientCtx.HomeDir)})
	assert.ErrorContains(t, err, "failed to migrate config")

	// try to migrate from client.toml - it should work and give us a big diff
	out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd.MigrateCommand(), []string{"v0.46", fmt.Sprintf("%s/config/client.toml", clientCtx.HomeDir), "--skip-validate", "--verbose"})
	assert.NilError(t, err)
	assert.Assert(t, strings.Contains(out.String(), "add app-db-backend key"))
}
