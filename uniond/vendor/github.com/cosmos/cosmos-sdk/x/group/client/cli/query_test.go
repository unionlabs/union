package cli_test

import (
	"context"
	"fmt"
	"io"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/x/group/client/cli"
)

func (s *CLITestSuite) TestQueryGroupInfo() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "invalid id",
			args:         []string{"invalid id"},
			expCmdOutput: `[invalid id]`,
		},
		{
			name:         "json output",
			args:         []string{"1", fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: `[1 --output=json]`,
		},
		{
			name:         "text output",
			args:         []string{"1", fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: `[1 --output=text]`,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryGroupInfoCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "group-info [id] [] [] Query for group info by group id")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryGroupPolicyInfo() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=json", accounts[0].Address.String()),
		},
		{
			name:         "text output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=text", accounts[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryGroupPolicyInfoCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "group-policy-info [group-policy-account] [] [] Query for group policy info by account address of group policy")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryGroupMembers() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{"1", fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: "1 --output=json",
		},
		{
			name:         "text output",
			args:         []string{"1", fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: "1 --output=text",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryGroupMembersCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "group-members [id] [] [] Query for group members by group id with pagination flags")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryGroupsByAdmin() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=json", accounts[0].Address.String()),
		},
		{
			name:         "text output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=text", accounts[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryGroupsByAdminCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "groups-by-admin [admin] [] [] Query for groups by admin account address with pagination flags")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryGroupPoliciesByGroup() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{"1", fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: "1 --output=json",
		},
		{
			name:         "text output",
			args:         []string{"1", fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: "1 --output=text",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryGroupPoliciesByGroupCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "group-policies-by-group [group-id] [] [] Query for group policies by group id with pagination flags")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryGroupPoliciesByAdmin() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=json", accounts[0].Address.String()),
		},
		{
			name:         "text output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=text", accounts[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryGroupPoliciesByAdminCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "group-policies-by-admin [admin] [] [] Query for group policies by admin account address with pagination flags")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryProposal() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{"1", fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: "1 --output=json",
		},
		{
			name:         "text output",
			args:         []string{"1", fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: "1 --output=text",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryProposalCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "proposal [id] [] [] Query for proposal by id")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryProposalsByGroupPolicy() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=json", accounts[0].Address.String()),
		},
		{
			name:         "text output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=text", accounts[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryProposalsByGroupPolicyCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "proposals-by-group-policy [group-policy-account] [] [] Query for proposals by account address of group policy with pagination flags")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryVoteByProposalVoter() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{"1", accounts[0].Address.String(), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("1 %s --output=json", accounts[0].Address.String()),
		},
		{
			name:         "text output",
			args:         []string{"1", accounts[0].Address.String(), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("1 %s --output=text", accounts[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryVoteByProposalVoterCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "vote [proposal-id] [voter] [] [] Query for vote by proposal id and voter account address")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryVotesByProposal() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{"1", fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: "1 --output=json",
		},
		{
			name:         "text output",
			args:         []string{"1", fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: "1 --output=text",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryVotesByProposalCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "votes-by-proposal [proposal-id] [] [] Query for votes by proposal id with pagination flags")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryTallyResult() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{"1", fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: "1 --output=json",
		},
		{
			name:         "text output",
			args:         []string{"1", fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: "1 --output=text",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryTallyResultCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "tally-result [proposal-id] [] [] Query tally result of proposal")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryVotesByVoter() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=json", accounts[0].Address.String()),
		},
		{
			name:         "text output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=text", accounts[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryVotesByVoterCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "votes-by-voter [voter] [] [] Query for votes by voter account address with pagination flags")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}

func (s *CLITestSuite) TestQueryGroupsByMembers() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name:         "json output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=json", accounts[0].Address.String()),
		},
		{
			name:         "text output",
			args:         []string{accounts[0].Address.String(), fmt.Sprintf("--%s=text", flags.FlagOutput)},
			expCmdOutput: fmt.Sprintf("%s --output=text", accounts[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.QueryGroupsByMemberCmd()

			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetOut(io.Discard)
			s.Require().NotNil(cmd)

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			s.Require().Contains(fmt.Sprint(cmd), "groups-by-member [address] [] [] Query for groups by member address with pagination flags")
			s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
		})
	}
}
