package cli_test

import (
	"bytes"
	"context"
	"fmt"
	"io"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/bank/client/cli"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

func (s *CLITestSuite) TestGetBalancesCmd() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	cmd := cli.GetBalancesCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expectResult proto.Message
		expectErr    bool
	}{
		{
			"valid query",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QueryAllBalancesResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				accounts[0].Address.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			&types.QueryAllBalancesResponse{},
			false,
		},
		{
			"valid query with denom",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QueryBalanceResponse{
					Balance: &sdk.Coin{},
				})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				accounts[0].Address.String(),
				fmt.Sprintf("--%s=photon", cli.FlagDenom),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			&sdk.Coin{},
			false,
		},
		{
			"invalid Address",
			func() client.Context {
				return s.baseCtx
			},
			[]string{
				"foo",
			},
			nil,
			true,
		},
		{
			"invalid denom",
			func() client.Context {
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Code: 1,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				accounts[0].Address.String(),
				fmt.Sprintf("--%s=foo", cli.FlagDenom),
			},
			nil,
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			var outBuf bytes.Buffer

			clientCtx := tc.ctxGen().WithOutput(&outBuf)
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(clientCtx, cmd))

			err := cmd.Execute()
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(s.encCfg.Codec.UnmarshalJSON(outBuf.Bytes(), tc.expectResult))
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetSpendableBalancesCmd() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	cmd := cli.GetSpendableBalancesCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expectResult proto.Message
		expectErr    bool
	}{
		{
			"valid query",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QuerySpendableBalancesResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				accounts[0].Address.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			&types.QuerySpendableBalancesResponse{},
			false,
		},
		{
			"valid query with denom flag",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QuerySpendableBalanceByDenomRequest{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				accounts[0].Address.String(),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
				fmt.Sprintf("--%s=photon", cli.FlagDenom),
			},
			&types.QuerySpendableBalanceByDenomResponse{},
			false,
		},
		{
			"invalid Address",
			func() client.Context {
				return s.baseCtx
			},
			[]string{
				"foo",
			},
			nil,
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			var outBuf bytes.Buffer

			clientCtx := tc.ctxGen().WithOutput(&outBuf)
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(clientCtx, cmd))

			err := cmd.Execute()
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(s.encCfg.Codec.UnmarshalJSON(outBuf.Bytes(), tc.expectResult))
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdDenomsMetadata() {
	cmd := cli.GetCmdDenomsMetadata()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expectResult proto.Message
		expectErr    bool
	}{
		{
			"valid query",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QueryDenomsMetadataResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			&types.QueryDenomsMetadataResponse{},
			false,
		},
		{
			"valid query with denom",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QueryDenomMetadataResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				fmt.Sprintf("--%s=photon", cli.FlagDenom),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			&types.QueryDenomMetadataResponse{},
			false,
		},
		{
			"invalid query with denom",
			func() client.Context {
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Code: 1,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				fmt.Sprintf("--%s=foo", cli.FlagDenom),
			},
			nil,
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			var outBuf bytes.Buffer

			clientCtx := tc.ctxGen().WithOutput(&outBuf)
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(clientCtx, cmd))

			err := cmd.Execute()
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(s.encCfg.Codec.UnmarshalJSON(outBuf.Bytes(), tc.expectResult))
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQueryTotalSupply() {
	cmd := cli.GetCmdQueryTotalSupply()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expectResult proto.Message
		expectErr    bool
	}{
		{
			"valid query",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QueryTotalSupplyResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			&types.QueryTotalSupplyResponse{},
			false,
		},
		{
			"valid query with denom",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QuerySupplyOfResponse{
					Amount: sdk.Coin{},
				})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				fmt.Sprintf("--%s=photon", cli.FlagDenom),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			&sdk.Coin{},
			false,
		},
		{
			"invalid query with denom",
			func() client.Context {
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Code: 1,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				fmt.Sprintf("--%s=foo", cli.FlagDenom),
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			nil,
			true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			var outBuf bytes.Buffer

			clientCtx := tc.ctxGen().WithOutput(&outBuf)
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(clientCtx, cmd))

			err := cmd.Execute()
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(s.encCfg.Codec.UnmarshalJSON(outBuf.Bytes(), tc.expectResult))
				s.Require().NoError(err)
			}
		})
	}
}

func (s *CLITestSuite) TestGetCmdQuerySendEnabled() {
	cmd := cli.GetCmdQuerySendEnabled()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expectResult proto.Message
		expectErr    bool
	}{
		{
			"valid query",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QuerySendEnabledResponse{
					SendEnabled: []*types.SendEnabled{},
				})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			&types.QuerySendEnabledResponse{},
			false,
		},
		{
			"valid query with denoms",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&types.QuerySendEnabledResponse{
					SendEnabled: []*types.SendEnabled{},
				})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			[]string{
				"photon",
				"stake",
				fmt.Sprintf("--%s=json", flags.FlagOutput),
			},
			&types.QuerySendEnabledResponse{},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		s.Run(tc.name, func() {
			var outBuf bytes.Buffer

			clientCtx := tc.ctxGen().WithOutput(&outBuf)
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(clientCtx, cmd))

			err := cmd.Execute()
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(s.encCfg.Codec.UnmarshalJSON(outBuf.Bytes(), tc.expectResult))
				s.Require().NoError(err)
			}
		})
	}
}
