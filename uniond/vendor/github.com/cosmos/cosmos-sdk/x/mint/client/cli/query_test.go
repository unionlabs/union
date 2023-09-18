package cli_test

import (
	"context"
	"fmt"
	"io"
	"strings"
	"testing"

	rpcclientmock "github.com/cometbft/cometbft/rpc/client/mock"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"

	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	testutilmod "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/mint"
	mintcli "github.com/cosmos/cosmos-sdk/x/mint/client/cli"
)

func TestGetCmdQueryParams(t *testing.T) {
	encCfg := testutilmod.MakeTestEncodingConfig(mint.AppModuleBasic{})
	kr := keyring.NewInMemory(encCfg.Codec)
	baseCtx := client.Context{}.
		WithKeyring(kr).
		WithTxConfig(encCfg.TxConfig).
		WithCodec(encCfg.Codec).
		WithClient(clitestutil.MockTendermintRPC{Client: rpcclientmock.Client{}}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain")

	cmd := mintcli.GetCmdQueryParams()

	testCases := []struct {
		name           string
		flagArgs       []string
		expCmdOutput   string
		expectedOutput string
	}{
		{
			"json output",
			[]string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			`[--height=1 --output=json]`,
			`{"mint_denom":"","inflation_rate_change":"0","inflation_max":"0","inflation_min":"0","goal_bonded":"0","blocks_per_year":"0"}`,
		},
		{
			"text output",
			[]string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			`[--height=1 --output=text]`,
			`blocks_per_year: "0"
goal_bonded: "0"
inflation_max: "0"
inflation_min: "0"
inflation_rate_change: "0"
mint_denom: ""`,
		},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			require.NotNil(t, cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.flagArgs)

			require.NoError(t, client.SetCmdClientContextHandler(baseCtx, cmd))

			if len(tc.flagArgs) != 0 {
				require.Contains(t, fmt.Sprint(cmd), "params [] [] Query the current minting parameters")
				require.Contains(t, fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(baseCtx, cmd, tc.flagArgs)
			require.NoError(t, err)
			require.Equal(t, tc.expectedOutput, strings.TrimSpace(out.String()))
		})
	}
}

func TestGetCmdQueryInflation(t *testing.T) {
	encCfg := testutilmod.MakeTestEncodingConfig(mint.AppModuleBasic{})
	kr := keyring.NewInMemory(encCfg.Codec)
	baseCtx := client.Context{}.
		WithKeyring(kr).
		WithTxConfig(encCfg.TxConfig).
		WithCodec(encCfg.Codec).
		WithClient(clitestutil.MockTendermintRPC{Client: rpcclientmock.Client{}}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain")

	cmd := mintcli.GetCmdQueryInflation()

	testCases := []struct {
		name           string
		flagArgs       []string
		expCmdOutput   string
		expectedOutput string
	}{
		{
			"json output",
			[]string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			`[--height=1 --output=json]`,
			`<nil>`,
		},
		{
			"text output",
			[]string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			`[--height=1 --output=text]`,
			`<nil>`,
		},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			require.NotNil(t, cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.flagArgs)

			require.NoError(t, client.SetCmdClientContextHandler(baseCtx, cmd))

			if len(tc.flagArgs) != 0 {
				require.Contains(t, fmt.Sprint(cmd), "inflation [] [] Query the current minting inflation value")
				require.Contains(t, fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(baseCtx, cmd, tc.flagArgs)
			require.NoError(t, err)
			require.Equal(t, tc.expectedOutput, strings.TrimSpace(out.String()))
		})
	}
}

func TestGetCmdQueryAnnualProvisions(t *testing.T) {
	encCfg := testutilmod.MakeTestEncodingConfig(mint.AppModuleBasic{})
	kr := keyring.NewInMemory(encCfg.Codec)
	baseCtx := client.Context{}.
		WithKeyring(kr).
		WithTxConfig(encCfg.TxConfig).
		WithCodec(encCfg.Codec).
		WithClient(clitestutil.MockTendermintRPC{Client: rpcclientmock.Client{}}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain")

	cmd := mintcli.GetCmdQueryAnnualProvisions()

	testCases := []struct {
		name           string
		flagArgs       []string
		expCmdOutput   string
		expectedOutput string
	}{
		{
			"json output",
			[]string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			`[--height=1 --output=json]`,
			`<nil>`,
		},
		{
			"text output",
			[]string{fmt.Sprintf("--%s=1", flags.FlagHeight), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			`[--height=1 --output=text]`,
			`<nil>`,
		},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			require.NotNil(t, cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.flagArgs)

			require.NoError(t, client.SetCmdClientContextHandler(baseCtx, cmd))

			if len(tc.flagArgs) != 0 {
				require.Contains(t, fmt.Sprint(cmd), "annual-provisions [] [] Query the current minting annual provisions value")
				require.Contains(t, fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(baseCtx, cmd, tc.flagArgs)
			require.NoError(t, err)
			require.Equal(t, tc.expectedOutput, strings.TrimSpace(out.String()))
		})
	}
}
