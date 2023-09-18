package cli_test

import (
	"fmt"

	"github.com/cosmos/cosmos-sdk/client/flags"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	"github.com/cosmos/cosmos-sdk/x/feegrant"
	"github.com/cosmos/cosmos-sdk/x/feegrant/client/cli"
)

func (s *CLITestSuite) TestCmdGetFeeGrant() {
	granter := s.addedGranter
	grantee := s.addedGrantee

	testCases := []struct {
		name         string
		args         []string
		expectErrMsg string
		expectErr    bool
		respType     *feegrant.QueryAllowanceResponse
		resp         *feegrant.Grant
	}{
		{
			"wrong granter",
			[]string{
				"wrong_granter",
				grantee.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"decoding bech32 failed",
			true, nil, nil,
		},
		{
			"wrong grantee",
			[]string{
				granter.String(),
				"wrong_grantee",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			"decoding bech32 failed",
			true, nil, nil,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryFeeGrant()
			out, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.args)

			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err)
				s.Require().NoError(s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.respType), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestCmdGetFeeGrantsByGrantee() {
	grantee := s.addedGrantee
	clientCtx := s.clientCtx

	testCases := []struct {
		name         string
		args         []string
		expectErr    bool
		resp         *feegrant.QueryAllowancesResponse
		expectLength int
	}{
		{
			"wrong grantee",
			[]string{
				"wrong_grantee",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true, nil, 0,
		},
		{
			"valid req",
			[]string{
				grantee.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false, &feegrant.QueryAllowancesResponse{}, 1,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryFeeGrantsByGrantee()

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)

			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.resp), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestCmdGetFeeGrantsByGranter() {
	granter := s.addedGranter
	clientCtx := s.clientCtx

	testCases := []struct {
		name         string
		args         []string
		expectErr    bool
		resp         *feegrant.QueryAllowancesByGranterResponse
		expectLength int
	}{
		{
			"wrong grantee",
			[]string{
				"wrong_grantee",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true, nil, 0,
		},
		{
			"valid req",
			[]string{
				granter.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false, &feegrant.QueryAllowancesByGranterResponse{}, 1,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryFeeGrantsByGranter()
			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)

			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.resp), out.String())
			}
		})
	}
}
