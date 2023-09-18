package client_test

import (
	"context"
	"fmt"
	"testing"

	"github.com/spf13/cobra"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/testutil"
)

func TestValidateCmd(t *testing.T) {
	// setup root and subcommands
	rootCmd := &cobra.Command{
		Use: "root",
	}
	queryCmd := &cobra.Command{
		Use: "query",
	}
	rootCmd.AddCommand(queryCmd)

	// command being tested
	distCmd := &cobra.Command{
		Use:                        "distr",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
	}
	queryCmd.AddCommand(distCmd)

	commissionCmd := &cobra.Command{
		Use: "commission",
	}
	distCmd.AddCommand(commissionCmd)

	tests := []struct {
		reason  string
		args    []string
		wantErr bool
	}{
		{"misspelled command", []string{"COMMISSION"}, true},
		{"no command provided", []string{}, false},
		{"help flag", []string{"COMMISSION", "--help"}, false},
		{"shorthand help flag", []string{"COMMISSION", "-h"}, false},
		{"flag only, no command provided", []string{"--gas", "1000atom"}, false},
		{"flag and misspelled command", []string{"--gas", "1000atom", "COMMISSION"}, true},
	}

	for _, tt := range tests {
		err := client.ValidateCmd(distCmd, tt.args)
		require.Equal(t, tt.wantErr, err != nil, tt.reason)
	}
}

func TestSetCmdClientContextHandler(t *testing.T) {
	initClientCtx := client.Context{}.WithHomeDir("/foo/bar").WithChainID("test-chain").WithKeyringDir("/foo/bar")

	newCmd := func() *cobra.Command {
		c := &cobra.Command{
			PreRunE: func(cmd *cobra.Command, args []string) error {
				return client.SetCmdClientContextHandler(initClientCtx, cmd)
			},
			RunE: func(cmd *cobra.Command, _ []string) error {
				_, err := client.GetClientTxContext(cmd)
				return err
			},
		}

		c.Flags().String(flags.FlagChainID, "", "network chain ID")
		c.Flags().String(flags.FlagHome, "", "home dir")

		return c
	}

	testCases := []struct {
		name            string
		expectedContext client.Context
		args            []string
	}{
		{
			"no flags set",
			initClientCtx,
			[]string{},
		},
		{
			"flags set",
			initClientCtx.WithChainID("new-chain-id"),
			[]string{
				fmt.Sprintf("--%s=new-chain-id", flags.FlagChainID),
			},
		},
		{
			"flags set with space",
			initClientCtx.WithHomeDir("/tmp/dir"),
			[]string{
				fmt.Sprintf("--%s", flags.FlagHome),
				"/tmp/dir",
			},
		},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			ctx := context.WithValue(context.Background(), client.ClientContextKey, &client.Context{})

			cmd := newCmd()
			_ = testutil.ApplyMockIODiscardOutErr(cmd)
			cmd.SetArgs(tc.args)

			require.NoError(t, cmd.ExecuteContext(ctx))

			clientCtx := client.GetClientContextFromCmd(cmd)
			require.Equal(t, tc.expectedContext, clientCtx)
		})
	}
}
