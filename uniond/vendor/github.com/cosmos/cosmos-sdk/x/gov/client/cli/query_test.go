package cli_test

import (
	"fmt"
	"strings"

	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/x/gov/client/cli"
)

func (s *CLITestSuite) TestCmdParams() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"json output",
			[]string{fmt.Sprintf("--%s=json", flags.FlagOutput)},
			"--output=json",
		},
		{
			"text output",
			[]string{fmt.Sprintf("--%s=text", flags.FlagOutput)},
			"--output=text",
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryParams()
			cmd.SetArgs(tc.args)

			s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
		})
	}
}

func (s *CLITestSuite) TestCmdParam() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"voting params",
			[]string{
				"voting",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			`voting --output=json`,
		},
		{
			"tally params",
			[]string{
				"tallying",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			`tallying --output=json`,
		},
		{
			"deposit params",
			[]string{
				"deposit",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			`deposit --output=json`,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryParam()
			cmd.SetArgs(tc.args)
			s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
		})
	}
}

func (s *CLITestSuite) TestCmdProposer() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"without proposal id",
			[]string{
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"--output=json",
		},
		{
			"with proposal id",
			[]string{
				"1",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"1 --output=json",
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryProposer()
			cmd.SetArgs(tc.args)
			s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
		})
	}
}

func (s *CLITestSuite) TestCmdTally() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"without proposal id",
			[]string{
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"--output=json",
		},
		{
			"with proposal id (json output)",
			[]string{
				"2",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"2 --output=json",
		},
		{
			"with proposal id (text output)",
			[]string{
				"1",
				fmt.Sprintf("--%s=text", flags.FlagOutput),
			},
			"1 --output=text",
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryTally()
			cmd.SetArgs(tc.args)
			s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
		})
	}
}

func (s *CLITestSuite) TestCmdGetProposal() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"get proposal with json response",
			[]string{
				"1",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"1 --output=json",
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryProposal()
			cmd.SetArgs(tc.args)
			s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
		})
	}
}

func (s *CLITestSuite) TestCmdGetProposals() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"get proposals as json response",
			[]string{
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"--output=json",
		},
		{
			"get proposals with invalid status",
			[]string{
				"--status=unknown",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"--status=unknown --output=json",
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryProposals()
			cmd.SetArgs(tc.args)
			s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
		})
	}
}

func (s *CLITestSuite) TestCmdQueryDeposits() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"get deposits",
			[]string{
				"10",
			},
			"10",
		},
		{
			"get deposits(json output)",
			[]string{
				"1",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"1 --output=json",
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryDeposits()
			cmd.SetArgs(tc.args)
			s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
		})
	}
}

func (s *CLITestSuite) TestCmdQueryDeposit() {
	val := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"get deposit with no depositer",
			[]string{
				"1",
			},
			"1",
		},
		{
			"get deposit with wrong deposit address",
			[]string{
				"1",
				"wrong address",
			},
			"1 wrong address",
		},
		{
			"get deposit (valid req)",
			[]string{
				"1",
				val[0].Address.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			fmt.Sprintf("1 %s --output=json", val[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryDeposit()
			cmd.SetArgs(tc.args)
			s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
		})
	}
}

func (s *CLITestSuite) TestCmdQueryVotes() {
	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"get votes with no proposal id",
			[]string{},
			"",
		},
		{
			"get votes of a proposal",
			[]string{
				"10",
			},
			"10",
		},
		{
			"get votes of a proposal (json output)",
			[]string{
				"1",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"1 --output=json",
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryVotes()
			cmd.SetArgs(tc.args)
			s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
		})
	}
}

func (s *CLITestSuite) TestCmdQueryVote() {
	val := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			"get vote of a proposal",
			[]string{
				"10",
				val[0].Address.String(),
			},
			fmt.Sprintf("10 %s", val[0].Address.String()),
		},
		{
			"get vote by wrong voter",
			[]string{
				"1",
				"wrong address",
			},
			"1 wrong address",
		},
		{
			"get vote of a proposal (json output)",
			[]string{
				"1",
				val[0].Address.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			fmt.Sprintf("1 %s --output=json", val[0].Address.String()),
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryVote()
			cmd.SetArgs(tc.args)

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), strings.TrimSpace(tc.expCmdOutput))
			}
		})
	}
}
