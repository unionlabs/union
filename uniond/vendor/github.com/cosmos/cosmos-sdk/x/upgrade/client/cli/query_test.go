package cli_test

import (
	"context"
	"fmt"
	"io"
	"testing"

	rpcclientmock "github.com/cometbft/cometbft/rpc/client/mock"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	testutilmod "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/upgrade"
	upgradecli "github.com/cosmos/cosmos-sdk/x/upgrade/client/cli"
)

func TestGetCurrentPlanCmd(t *testing.T) {
	encCfg := testutilmod.MakeTestEncodingConfig(upgrade.AppModuleBasic{})
	kr := keyring.NewInMemory(encCfg.Codec)
	baseCtx := client.Context{}.
		WithKeyring(kr).
		WithTxConfig(encCfg.TxConfig).
		WithCodec(encCfg.Codec).
		WithClient(clitestutil.MockTendermintRPC{Client: rpcclientmock.Client{}}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain")

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: `[--output=json]`,
		},
		{
			name:         "text output",
			args:         []string{fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: `[--output=text]`,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd := upgradecli.GetCurrentPlanCmd()
			cmd.SetOut(io.Discard)
			require.NotNil(t, cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			require.NoError(t, client.SetCmdClientContextHandler(baseCtx, cmd))

			require.Contains(t, fmt.Sprint(cmd), "plan [] [] get upgrade plan (if one exists)")
			require.Contains(t, fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func TestGetAppliedPlanCmd(t *testing.T) {
	encCfg := testutilmod.MakeTestEncodingConfig(upgrade.AppModuleBasic{})
	kr := keyring.NewInMemory(encCfg.Codec)
	baseCtx := client.Context{}.
		WithKeyring(kr).
		WithTxConfig(encCfg.TxConfig).
		WithCodec(encCfg.Codec).
		WithClient(clitestutil.MockTendermintRPC{Client: rpcclientmock.Client{}}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain")

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{"test-upgrade", fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: `[test-upgrade --output=json]`,
		},
		{
			name:         "text output",
			args:         []string{"test-upgrade", fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: `[test-upgrade --output=text]`,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd := upgradecli.GetAppliedPlanCmd()
			cmd.SetOut(io.Discard)
			require.NotNil(t, cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			require.NoError(t, client.SetCmdClientContextHandler(baseCtx, cmd))

			require.Contains(t, fmt.Sprint(cmd), "applied [upgrade-name] [] [] block header for height at which a completed upgrade was applied")
			require.Contains(t, fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func TestGetModuleVersionsCmd(t *testing.T) {
	encCfg := testutilmod.MakeTestEncodingConfig(upgrade.AppModuleBasic{})
	kr := keyring.NewInMemory(encCfg.Codec)
	baseCtx := client.Context{}.
		WithKeyring(kr).
		WithTxConfig(encCfg.TxConfig).
		WithCodec(encCfg.Codec).
		WithClient(clitestutil.MockTendermintRPC{Client: rpcclientmock.Client{}}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain")

	testCases := []struct {
		msg          string
		args         []string
		expCmdOutput string
	}{
		{
			msg:          "test full query with json output",
			args:         []string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: `--height=1 --output=json`,
		},
		{
			msg:          "test full query with text output",
			args:         []string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: `--height=1 --output=text`,
		},
		{
			msg:          "test single module",
			args:         []string{"bank", fmt.Sprintf("--%s=1", flags.FlagHeight)},
			expCmdOutput: `bank --height=1`,
		},
		{
			msg:          "test non-existent module",
			args:         []string{"abcdefg", fmt.Sprintf("--%s=1", flags.FlagHeight)},
			expCmdOutput: `abcdefg --height=1`,
		},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.msg, func(t *testing.T) {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd := upgradecli.GetModuleVersionsCmd()
			cmd.SetOut(io.Discard)
			require.NotNil(t, cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			require.NoError(t, client.SetCmdClientContextHandler(baseCtx, cmd))

			require.Contains(t, fmt.Sprint(cmd), "module_versions [optional module_name] [] [] get the list of module versions")
			require.Contains(t, fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}
