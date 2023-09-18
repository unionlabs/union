package cli_test

import (
	"fmt"
	"strings"

	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/cosmos-sdk/client/flags"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/client/cli"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

func (s *CLITestSuite) TestGetCmdQueryValidator() {
	testCases := []struct {
		name      string
		args      []string
		expectErr bool
	}{
		{
			"with invalid address ",
			[]string{"somethinginvalidaddress", fmt.Sprintf("--%s=json", flags.FlagOutput)},
			true,
		},
		{
			"happy case",
			[]string{sdk.ValAddress(s.addrs[0]).String(), fmt.Sprintf("--%s=json", flags.FlagOutput)},
			false,
		},
	}
	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryValidator()
			clientCtx := s.clientCtx
			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().NotEqual("internal", err.Error())
			} else {
				var result types.Validator
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), &result))
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryValidators() {
	testCases := []struct {
		name              string
		args              []string
		minValidatorCount int
	}{
		{
			"one validator case",
			[]string{
				fmt.Sprintf("--%s=json", flags.FlagOutput),
				fmt.Sprintf("--%s=1", flags.FlagLimit),
			},
			1,
		},
		{
			"multi validator case",
			[]string{fmt.Sprintf("--%s=json", flags.FlagOutput)},
			0,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryValidators()
			clientCtx := s.clientCtx

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			s.Require().NoError(err)

			var result types.QueryValidatorsResponse
			s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), &result))
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryDelegation() {
	testCases := []struct {
		name     string
		args     []string
		expErr   bool
		respType proto.Message
	}{
		{
			"with wrong delegator address",
			[]string{
				"wrongDelAddr",
				s.addrs[1].String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true, nil,
		},
		{
			"with wrong validator address",
			[]string{
				s.addrs[0].String(),
				"wrongValAddr",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true, nil,
		},
		{
			"with json output",
			[]string{
				s.addrs[0].String(),
				sdk.ValAddress(s.addrs[1]).String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
			&types.DelegationResponse{},
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryDelegation()
			clientCtx := s.clientCtx

			_, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			if tc.expErr {
				s.Require().Error(err)
			} else {
				s.Require().Contains(err.Error(), "Marshal called with nil")
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryDelegations() {
	testCases := []struct {
		name     string
		args     []string
		expErr   bool
		respType proto.Message
	}{
		{
			"with no delegator address",
			[]string{},
			true, nil,
		},
		{
			"with wrong delegator address",
			[]string{"wrongDelAddr"},
			true, nil,
		},
		{
			"valid request (height specific)",
			[]string{
				s.addrs[0].String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
				fmt.Sprintf("--%s=1", flags.FlagHeight),
			},
			false,
			&types.QueryDelegatorDelegationsResponse{},
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryDelegations()
			clientCtx := s.clientCtx

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			if tc.expErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.respType), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryValidatorDelegations() {
	testCases := []struct {
		name     string
		args     []string
		expErr   bool
		respType proto.Message
	}{
		{
			"with no validator address",
			[]string{},
			true, nil,
		},
		{
			"wrong validator address",
			[]string{"wrongValAddr"},
			true, nil,
		},
		{
			"valid request(height specific)",
			[]string{
				s.addrs[0].String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
				fmt.Sprintf("--%s=1", flags.FlagHeight),
			},
			false,
			&types.QueryValidatorDelegationsResponse{},
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryDelegations()
			clientCtx := s.clientCtx

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			if tc.expErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.respType), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryUnbondingDelegations() {
	testCases := []struct {
		name   string
		args   []string
		expErr bool
	}{
		{
			"wrong delegator address",
			[]string{
				"wrongDelAddr",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"valid request",
			[]string{
				s.addrs[0].String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryUnbondingDelegations()
			clientCtx := s.clientCtx

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)

			if tc.expErr {
				s.Require().Error(err)
			} else {
				var ubds types.QueryDelegatorUnbondingDelegationsResponse
				err = s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &ubds)

				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryUnbondingDelegation() {
	testCases := []struct {
		name   string
		args   []string
		expErr bool
	}{
		{
			"wrong delegator address",
			[]string{
				"wrongDelAddr",
				s.addrs[0].String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"wrong validator address",
			[]string{
				s.addrs[0].String(),
				"wrongValAddr",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"valid request",
			[]string{
				s.addrs[0].String(),
				sdk.ValAddress(s.addrs[1]).String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryUnbondingDelegation()
			clientCtx := s.clientCtx

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)

			if tc.expErr {
				s.Require().Error(err)
			} else {
				var ubd types.UnbondingDelegation

				err = s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &ubd)
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryValidatorUnbondingDelegations() {
	testCases := []struct {
		name   string
		args   []string
		expErr bool
	}{
		{
			"wrong validator address",
			[]string{
				"wrongValAddr",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"valid request",
			[]string{
				sdk.ValAddress(s.addrs[0]).String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryValidatorUnbondingDelegations()
			clientCtx := s.clientCtx

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)

			if tc.expErr {
				s.Require().Error(err)
			} else {
				var ubds types.QueryValidatorUnbondingDelegationsResponse
				err = s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &ubds)
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryRedelegations() {
	testCases := []struct {
		name   string
		args   []string
		expErr bool
	}{
		{
			"wrong delegator address",
			[]string{
				"wrongdeladdr",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"valid request",
			[]string{
				s.addrs[0].String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryRedelegations()
			clientCtx := s.clientCtx

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)

			if tc.expErr {
				s.Require().Error(err)
			} else {
				var redelegations types.QueryRedelegationsResponse
				err = s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &redelegations)
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryRedelegation() {
	testCases := []struct {
		name   string
		args   []string
		expErr bool
	}{
		{
			"wrong delegator address",
			[]string{
				"wrongdeladdr",
				sdk.ValAddress(s.addrs[0]).String(),
				sdk.ValAddress(s.addrs[1]).String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"wrong source validator address address",
			[]string{
				s.addrs[0].String(),
				"wrongSrcValAddress",
				sdk.ValAddress(s.addrs[1]).String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"wrong destination validator address address",
			[]string{
				s.addrs[0].String(),
				sdk.ValAddress(s.addrs[0]).String(),
				"wrongDestValAddress",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"valid request",
			[]string{
				s.addrs[0].String(),
				sdk.ValAddress(s.addrs[0]).String(),
				sdk.ValAddress(s.addrs[1]).String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryRedelegation()
			clientCtx := s.clientCtx

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)

			if tc.expErr {
				s.Require().Error(err)
			} else {
				var redelegations types.QueryRedelegationsResponse

				err = s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &redelegations)
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryValidatorRedelegations() {
	testCases := []struct {
		name   string
		args   []string
		expErr bool
	}{
		{
			"wrong validator address",
			[]string{
				"wrongValAddr",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"valid request",
			[]string{
				sdk.ValAddress(s.addrs[0]).String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryValidatorRedelegations()
			clientCtx := s.clientCtx

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)

			if tc.expErr {
				s.Require().Error(err)
			} else {
				var redelegations types.QueryRedelegationsResponse
				err = s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &redelegations)
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryPool() {
	testCases := []struct {
		name           string
		args           []string
		expectedOutput string
	}{
		{
			"with text",
			[]string{
				fmt.Sprintf("--%s=text", flags.FlagOutput),
				fmt.Sprintf("--%s=1", flags.FlagHeight),
			},
			`bonded_tokens: "0"
not_bonded_tokens: "0"`,
		},
		{
			"with json",
			[]string{
				fmt.Sprintf("--%s=json", flags.FlagOutput),
				fmt.Sprintf("--%s=1", flags.FlagHeight),
			},
			`{"not_bonded_tokens":"0","bonded_tokens":"0"}`,
		},
	}
	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryPool()
			clientCtx := s.clientCtx
			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			s.Require().NoError(err)
			s.Require().Equal(tc.expectedOutput, strings.TrimSpace(out.String()))
		})
	}
}
