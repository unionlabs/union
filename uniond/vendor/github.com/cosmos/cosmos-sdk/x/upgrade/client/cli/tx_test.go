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

func TestModuleVersionsCLI(t *testing.T) {
	cmd := upgradecli.GetModuleVersionsCmd()
	cmd.SetOut(io.Discard)
	require.NotNil(t, cmd)

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
		expCmdOuptut string
	}{
		{
			msg:          "test full query with json output",
			args:         []string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOuptut: `--height=1 --output=json`,
		},
		{
			msg:          "test full query with text output",
			args:         []string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOuptut: `--height=1 --output=text`,
		},
		{
			msg:          "test single module",
			args:         []string{"bank", fmt.Sprintf("--%s=1", flags.FlagHeight)},
			expCmdOuptut: `bank --height=1`,
		},
		{
			msg:          "test non-existent module",
			args:         []string{"abcdefg", fmt.Sprintf("--%s=1", flags.FlagHeight)},
			expCmdOuptut: `abcdefg --height=1`,
		},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.msg, func(t *testing.T) {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			require.NotNil(t, cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			require.NoError(t, client.SetCmdClientContextHandler(baseCtx, cmd))

			if len(tc.args) != 0 {
				require.Contains(t, fmt.Sprint(cmd), tc.expCmdOuptut)
			}
		})
	}
}
