package cli_test

import (
	"fmt"
	"time"

	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/authz"
	"github.com/cosmos/cosmos-sdk/x/authz/client/cli"
	authzclitestutil "github.com/cosmos/cosmos-sdk/x/authz/client/testutil"
)

func (s *CLITestSuite) TestQueryAuthorizations() {
	val := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	grantee := s.grantee[0]
	twoHours := time.Now().Add(time.Minute * time.Duration(120)).Unix()

	_, err := authzclitestutil.CreateGrant(
		s.clientCtx,
		[]string{
			grantee.String(),
			"send",
			fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
			fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
			fmt.Sprintf("--%s=%s", flags.FlagFrom, val[0].Address),
			fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
			fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
			fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(10))).String()),
		},
	)
	s.Require().NoError(err)

	testCases := []struct {
		name      string
		args      []string
		expectErr bool
		expErrMsg string
	}{
		{
			"Error: Invalid grantee",
			[]string{
				val[0].Address.String(),
				"invalid grantee",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
			"decoding bech32 failed: invalid character in string: ' '",
		},
		{
			"Error: Invalid granter",
			[]string{
				"invalid granter",
				grantee.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
			"decoding bech32 failed: invalid character in string: ' '",
		},
		{
			"Valid txn (json)",
			[]string{
				val[0].Address.String(),
				grantee.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
			``,
		},
	}
	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryGrants()
			resp, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.args)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(string(resp.Bytes()), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
				var grants authz.QueryGrantsResponse
				err = s.clientCtx.Codec.UnmarshalJSON(resp.Bytes(), &grants)
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestQueryAuthorization() {
	val := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	grantee := s.grantee[0]
	twoHours := time.Now().Add(time.Minute * time.Duration(120)).Unix()

	_, err := authzclitestutil.CreateGrant(
		s.clientCtx,
		[]string{
			grantee.String(),
			"send",
			fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
			fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
			fmt.Sprintf("--%s=%s", flags.FlagFrom, val[0].Address),
			fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
			fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
			fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(10))).String()),
		},
	)
	s.Require().NoError(err)

	testCases := []struct {
		name      string
		args      []string
		expectErr bool
	}{
		{
			"Error: Invalid grantee",
			[]string{
				val[0].Address.String(),
				"invalid grantee",
				typeMsgSend,
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"Error: Invalid granter",
			[]string{
				"invalid granter",
				grantee.String(),
				typeMsgSend,
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
		},
		{
			"Valid txn (json)",
			[]string{
				val[0].Address.String(),
				grantee.String(),
				typeMsgSend,
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
		},
		{
			"Valid txn with allowed list (json)",
			[]string{
				val[0].Address.String(),
				s.grantee[3].String(),
				typeMsgSend,
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
		},
	}
	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			cmd := cli.GetCmdQueryGrants()
			_, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.args)
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestQueryGranterGrants() {
	val := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	grantee := s.grantee[0]
	require := s.Require()

	testCases := []struct {
		name        string
		args        []string
		expectErr   bool
		expectedErr string
	}{
		{
			"invalid address",
			[]string{
				"invalid-address",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			true,
			"decoding bech32 failed",
		},
		{
			"no authorization found",
			[]string{
				grantee.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
			"",
		},
		{
			"valid case",
			[]string{
				val[0].Address.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
			"",
		},
		{
			"valid case with pagination",
			[]string{
				val[0].Address.String(),
				"--limit=2",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			false,
			"",
		},
	}
	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.GetQueryGranterGrants()
			out, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, tc.args)
			if tc.expectErr {
				require.Error(err)
				require.Contains(out.String(), tc.expectedErr)
			} else {
				require.NoError(err)
				var grants authz.QueryGranterGrantsResponse
				require.NoError(s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &grants))
			}
		})
	}
}
