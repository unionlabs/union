package cli_test

import (
	"context"
	"fmt"
	"io"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/bank/client/cli"
)

func (s *CLITestSuite) TestSendTxCmd() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)
	cmd := cli.NewSendTxCmd()
	cmd.SetOutput(io.Discard)

	extraArgs := []string{
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("photon", sdk.NewInt(10))).String()),
		fmt.Sprintf("--%s=test-chain", flags.FlagChainID),
	}

	testCases := []struct {
		name      string
		ctxGen    func() client.Context
		from, to  sdk.AccAddress
		amount    sdk.Coins
		extraArgs []string
		expectErr bool
	}{
		{
			"valid transaction",
			func() client.Context {
				return s.baseCtx
			},
			accounts[0].Address,
			accounts[0].Address,
			sdk.NewCoins(
				sdk.NewCoin("stake", sdk.NewInt(10)),
				sdk.NewCoin("photon", sdk.NewInt(40)),
			),
			extraArgs,
			false,
		},
		{
			"invalid to Address",
			func() client.Context {
				return s.baseCtx
			},
			accounts[0].Address,
			sdk.AccAddress{},
			sdk.NewCoins(
				sdk.NewCoin("stake", sdk.NewInt(10)),
				sdk.NewCoin("photon", sdk.NewInt(40)),
			),
			extraArgs,
			true,
		},
		{
			"invalid coins",
			func() client.Context {
				return s.baseCtx
			},
			accounts[0].Address,
			accounts[0].Address,
			nil,
			extraArgs,
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(append([]string{tc.from.String(), tc.to.String(), tc.amount.String()}, tc.extraArgs...))

			s.Require().NoError(client.SetCmdClientContextHandler(tc.ctxGen(), cmd))

			err := cmd.Execute()
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestMultiSendTxCmd() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 3)

	cmd := cli.NewMultiSendTxCmd()
	cmd.SetOutput(io.Discard)

	extraArgs := []string{
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("photon", sdk.NewInt(10))).String()),
		fmt.Sprintf("--%s=test-chain", flags.FlagChainID),
	}

	testCases := []struct {
		name      string
		ctxGen    func() client.Context
		from      string
		to        []string
		amount    sdk.Coins
		extraArgs []string
		expectErr bool
	}{
		{
			"valid transaction",
			func() client.Context {
				return s.baseCtx
			},
			accounts[0].Address.String(),
			[]string{
				accounts[1].Address.String(),
				accounts[2].Address.String(),
			},
			sdk.NewCoins(
				sdk.NewCoin("stake", sdk.NewInt(10)),
				sdk.NewCoin("photon", sdk.NewInt(40)),
			),
			extraArgs,
			false,
		},
		{
			"invalid from Address",
			func() client.Context {
				return s.baseCtx
			},
			"foo",
			[]string{
				accounts[1].Address.String(),
				accounts[2].Address.String(),
			},
			sdk.NewCoins(
				sdk.NewCoin("stake", sdk.NewInt(10)),
				sdk.NewCoin("photon", sdk.NewInt(40)),
			),
			extraArgs,
			true,
		},
		{
			"invalid recipients",
			func() client.Context {
				return s.baseCtx
			},
			accounts[0].Address.String(),
			[]string{
				accounts[1].Address.String(),
				"bar",
			},
			sdk.NewCoins(
				sdk.NewCoin("stake", sdk.NewInt(10)),
				sdk.NewCoin("photon", sdk.NewInt(40)),
			),
			extraArgs,
			true,
		},
		{
			"invalid amount",
			func() client.Context {
				return s.baseCtx
			},
			accounts[0].Address.String(),
			[]string{
				accounts[1].Address.String(),
				accounts[2].Address.String(),
			},
			nil,
			extraArgs,
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			var args []string
			args = append(args, tc.from)
			args = append(args, tc.to...)
			args = append(args, tc.amount.String())
			args = append(args, tc.extraArgs...)

			cmd.SetContext(ctx)
			cmd.SetArgs(args)

			s.Require().NoError(client.SetCmdClientContextHandler(tc.ctxGen(), cmd))

			err := cmd.Execute()
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}
