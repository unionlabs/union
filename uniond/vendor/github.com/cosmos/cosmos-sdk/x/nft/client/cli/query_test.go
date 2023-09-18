package cli_test

import (
	"context"
	"fmt"
	"io"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	"github.com/cosmos/cosmos-sdk/x/nft"
	"github.com/cosmos/cosmos-sdk/x/nft/client/cli"
)

func (s *CLITestSuite) TestQueryClass() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{testClassID, fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: `[kitty --output=json]`,
		},
		{
			name:         "text output",
			args:         []string{testClassID, fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: `[kitty --output=text]`,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryClass()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "class [class-id] [] [] query an NFT class based on its id")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)

			_, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.args)
			s.Require().NoError(err)
		})
	}
}

func (s *CLITestSuite) TestQueryClasses() {
	testCases := []struct {
		name         string
		flagArgs     []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			flagArgs:     []string{fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: `[--output=json]`,
		},
		{
			name:         "text output",
			flagArgs:     []string{fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: `[--output=text]`,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryClasses()
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.flagArgs)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "classes [] [] query all NFT classes")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)

			_, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.flagArgs)
			s.Require().NoError(err)
		})
	}
}

func (s *CLITestSuite) TestQueryNFT() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{testClassID, testID, fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: `[kitty kitty1 --output=json]`,
		},
		{
			name:         "text output",
			args:         []string{testClassID, testID, fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: `[kitty kitty1 --output=text]`,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryNFT()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "nft [class-id] [nft-id] [] [] query an NFT based on its class and id")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)

			_, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.args)
			s.Require().NoError(err)
		})
	}
}

func (s *CLITestSuite) TestQueryNFTs() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name string
		args struct {
			ClassID string
			Owner   string
		}
		expectErr bool
		expErrMsg string
	}{
		{
			name: "empty class id and owner",
			args: struct {
				ClassID string
				Owner   string
			}{},
			expectErr: true,
			expErrMsg: "must provide at least one of classID or owner",
		},
		{
			name: "valid case",
			args: struct {
				ClassID string
				Owner   string
			}{
				ClassID: testClassID,
				Owner:   accounts[0].Address.String(),
			},
			expectErr: false,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryNFTs()
			var args []string
			args = append(args, fmt.Sprintf("--%s=%s", cli.FlagClassID, tc.args.ClassID))
			args = append(args, fmt.Sprintf("--%s=%s", cli.FlagOwner, tc.args.Owner))
			args = append(args, fmt.Sprintf("--%s=json", flags.FlagOutput))

			out, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, args)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
				var result nft.QueryNFTsResponse
				err = s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &result)
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestQueryOwner() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{testClassID, testID, fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: `[kitty kitty1 --output=json]`,
		},
		{
			name:         "text output",
			args:         []string{testClassID, testID, fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: `[kitty kitty1 --output=text]`,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryOwner()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "owner [class-id] [nft-id] [] [] query the owner of the NFT based on its class and id")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)

			_, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.args)
			s.Require().NoError(err)
		})
	}
}

func (s *CLITestSuite) TestQueryBalance() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{accounts[0].Address.String(), testClassID, fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s kitty --output=json", accounts[0].Address.String()),
		},
		{
			name:         "text output",
			args:         []string{accounts[0].Address.String(), testClassID, fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s kitty --output=text", accounts[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryBalance()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "balance [owner] [class-id] [] [] query the number of NFTs of a given class owned by the owner")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)

			_, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.args)
			s.Require().NoError(err)
		})
	}
}

func (s *CLITestSuite) TestQuerySupply() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "valid case",
			args:         []string{testClassID, fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: `[kitty --output=json]`,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQuerySupply()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "supply [class-id] [] [] query the number of nft based on the class")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)

			_, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.args)
			s.Require().NoError(err)
		})
	}
}
