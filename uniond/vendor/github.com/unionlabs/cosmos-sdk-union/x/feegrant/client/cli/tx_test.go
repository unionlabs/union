package cli_test

import (
	"fmt"
	"io"
	"strings"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/suite"

	_ "cosmossdk.io/api/cosmos/feegrant/v1beta1"
	v1 "cosmossdk.io/api/cosmos/gov/v1"
	v1beta1 "cosmossdk.io/api/cosmos/gov/v1beta1"
	"cosmossdk.io/core/address"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/client/cli"
	"cosmossdk.io/x/feegrant/module"
	"cosmossdk.io/x/gov"
	govcli "cosmossdk.io/x/gov/client/cli"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	sdk "github.com/cosmos/cosmos-sdk/types"
	testutilmod "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

const (
	oneYear  = 365 * 24 * 60 * 60
	tenHours = 10 * 60 * 60
	oneHour  = 60 * 60
)

type CLITestSuite struct {
	suite.Suite

	addedGranter sdk.AccAddress
	addedGrantee sdk.AccAddress
	addedGrant   feegrant.Grant

	kr        keyring.Keyring
	baseCtx   client.Context
	encCfg    testutilmod.TestEncodingConfig
	clientCtx client.Context

	accounts []sdk.AccAddress
}

func TestCLITestSuite(t *testing.T) {
	suite.Run(t, new(CLITestSuite))
}

func (s *CLITestSuite) SetupSuite() {
	s.T().Log("setting up integration test suite")

	s.encCfg = testutilmod.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{}, gov.AppModule{})
	s.kr = keyring.NewInMemory(s.encCfg.Codec)
	s.baseCtx = client.Context{}.
		WithKeyring(s.kr).
		WithTxConfig(s.encCfg.TxConfig).
		WithCodec(s.encCfg.Codec).
		WithClient(clitestutil.MockCometRPC{}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain").
		WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
		WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
		WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))

	ctxGen := func() client.Context {
		bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
		c := clitestutil.NewMockCometRPCWithResponseQueryValue(bz)

		return s.baseCtx.WithClient(c)
	}
	s.clientCtx = ctxGen()

	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 2)

	granter := accounts[0].Address
	grantee := accounts[1].Address
	s.createGrant(granter, grantee, s.baseCtx.AddressCodec)

	granteeStr, err := s.baseCtx.AddressCodec.BytesToString(grantee)
	s.Require().NoError(err)
	granterStr, err := s.baseCtx.AddressCodec.BytesToString(granter)
	s.Require().NoError(err)

	grant, err := feegrant.NewGrant(granterStr, granteeStr, &feegrant.BasicAllowance{
		SpendLimit: sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100))),
	})
	s.Require().NoError(err)

	s.addedGrant = grant
	s.addedGranter = granter
	s.addedGrantee = grantee
	for _, v := range accounts {
		s.accounts = append(s.accounts, v.Address)
	}
	s.accounts[1] = accounts[1].Address
}

// createGrant creates a new basic allowance fee grant from granter to grantee.
func (s *CLITestSuite) createGrant(granter, grantee sdk.AccAddress, addressCodec address.Codec) {
	commonFlags := []string{
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(100))).String()),
	}

	fee := sdk.NewCoin("stake", sdkmath.NewInt(100))

	granterAddr, err := addressCodec.BytesToString(granter)
	s.Require().NoError(err)
	granteeAddr, err := addressCodec.BytesToString(grantee)
	s.Require().NoError(err)

	args := append(
		[]string{
			granterAddr,
			granteeAddr,
			fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, fee.String()),
			fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
			fmt.Sprintf("--%s=%s", cli.FlagExpiration, getFormattedExpiration(oneYear)),
		},
		commonFlags...,
	)

	cmd := cli.NewCmdFeeGrant()
	out, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, args)
	s.Require().NoError(err)

	var resp sdk.TxResponse
	s.Require().NoError(s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &resp), out.String())
	s.Require().Equal(resp.Code, uint32(0))
}

func (s *CLITestSuite) TestNewCmdFeeGrant() {
	granter := s.accounts[0]
	clientCtx := s.clientCtx
	granterAddr, err := s.baseCtx.AddressCodec.BytesToString(granter)
	s.Require().NoError(err)
	alreadyExistedGranteeAddr, err := s.baseCtx.AddressCodec.BytesToString(s.addedGrantee)
	s.Require().NoError(err)
	fromAddr, fromName, _, err := client.GetFromFields(s.baseCtx, s.kr, granterAddr)
	s.Require().Equal(fromAddr, granter)
	s.Require().NoError(err)

	commonFlags := []string{
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
	}

	testCases := []struct {
		name         string
		args         []string
		expectErr    bool
		expectedCode uint32
		respType     proto.Message
	}{
		{
			"wrong granter address",
			append(
				[]string{
					"wrong_granter",
					"cosmos1nph3cfzk6trsmfxkeu943nvach5qw4vwstnvkl",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			true, 0, nil,
		},
		{
			"wrong grantee address",
			append(
				[]string{
					granterAddr,
					"wrong_grantee",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			true, 0, nil,
		},
		{
			"wrong granter key name",
			append(
				[]string{
					"invalid_granter",
					"cosmos16dun6ehcc86e03wreqqww89ey569wuj4em572w",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			true, 0, nil,
		},
		{
			"valid basic fee grant",
			append(
				[]string{
					granterAddr,
					"cosmos1nph3cfzk6trsmfxkeu943nvach5qw4vwstnvkl",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"valid basic fee grant with granter key name",
			append(
				[]string{
					fromName,
					"cosmos16dun6ehcc86e03wreqqww89ey569wuj4em572w",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, fromName),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"valid basic fee grant with amino",
			append(
				[]string{
					granterAddr,
					"cosmos1v57fx2l2rt6ehujuu99u2fw05779m5e2ux4z2h",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"valid basic fee grant without spend limit",
			append(
				[]string{
					granterAddr,
					"cosmos17h5lzptx3ghvsuhk7wx4c4hnl7rsswxjer97em",
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"valid basic fee grant without expiration",
			append(
				[]string{
					granterAddr,
					"cosmos16dlc38dcqt0uralyd8hksxyrny6kaeqfjvjwp5",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"valid basic fee grant without spend-limit and expiration",
			append(
				[]string{
					granterAddr,
					"cosmos1ku40qup9vwag4wtf8cls9mkszxfthaklxkp3c8",
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"try to add existed grant",
			append(
				[]string{
					granterAddr,
					alreadyExistedGranteeAddr,
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			false, 18, &sdk.TxResponse{},
		},
		{
			"invalid number of args(periodic fee grant)",
			append(
				[]string{
					granterAddr,
					"cosmos1nph3cfzk6trsmfxkeu943nvach5qw4vwstnvkl",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%s", cli.FlagPeriodLimit, "10stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
					fmt.Sprintf("--%s=%s", cli.FlagExpiration, getFormattedExpiration(tenHours)),
				},
				commonFlags...,
			),
			true, 0, nil,
		},
		{
			"period mentioned and period limit omitted, invalid periodic grant",
			append(
				[]string{
					granterAddr,
					"cosmos1nph3cfzk6trsmfxkeu943nvach5qw4vwstnvkl",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%d", cli.FlagPeriod, tenHours),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
					fmt.Sprintf("--%s=%s", cli.FlagExpiration, getFormattedExpiration(oneHour)),
				},
				commonFlags...,
			),
			true, 0, nil,
		},
		{
			"period cannot be greater than the actual expiration(periodic fee grant)",
			append(
				[]string{
					granterAddr,
					"cosmos1nph3cfzk6trsmfxkeu943nvach5qw4vwstnvkl",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%d", cli.FlagPeriod, tenHours),
					fmt.Sprintf("--%s=%s", cli.FlagPeriodLimit, "10stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
					fmt.Sprintf("--%s=%s", cli.FlagExpiration, getFormattedExpiration(oneHour)),
				},
				commonFlags...,
			),
			true, 0, nil,
		},
		{
			"valid periodic fee grant",
			append(
				[]string{
					granterAddr,
					"cosmos1w55kgcf3ltaqdy4ww49nge3klxmrdavrr6frmp",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%d", cli.FlagPeriod, oneHour),
					fmt.Sprintf("--%s=%s", cli.FlagPeriodLimit, "10stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
					fmt.Sprintf("--%s=%s", cli.FlagExpiration, getFormattedExpiration(tenHours)),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"valid periodic fee grant without spend-limit",
			append(
				[]string{
					granterAddr,
					"cosmos1vevyks8pthkscvgazc97qyfjt40m6g9xe85ry8",
					fmt.Sprintf("--%s=%d", cli.FlagPeriod, oneHour),
					fmt.Sprintf("--%s=%s", cli.FlagPeriodLimit, "10stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
					fmt.Sprintf("--%s=%s", cli.FlagExpiration, getFormattedExpiration(tenHours)),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"valid periodic fee grant without expiration",
			append(
				[]string{
					granterAddr,
					"cosmos14cm33pvnrv2497tyt8sp9yavhmw83nwej3m0e8",
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
					fmt.Sprintf("--%s=%d", cli.FlagPeriod, oneHour),
					fmt.Sprintf("--%s=%s", cli.FlagPeriodLimit, "10stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"valid periodic fee grant without spend-limit and expiration",
			append(
				[]string{
					granterAddr,
					"cosmos12nyk4pcf4arshznkpz882e4l4ts0lt0ap8ce54",
					fmt.Sprintf("--%s=%d", cli.FlagPeriod, oneHour),
					fmt.Sprintf("--%s=%s", cli.FlagPeriodLimit, "10stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			false, 0, &sdk.TxResponse{},
		},
		{
			"invalid expiration",
			append(
				[]string{
					granterAddr,
					"cosmos1vevyks8pthkscvgazc97qyfjt40m6g9xe85ry8",
					fmt.Sprintf("--%s=%d", cli.FlagPeriod, oneHour),
					fmt.Sprintf("--%s=%s", cli.FlagPeriodLimit, "10stake"),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
					fmt.Sprintf("--%s=%s", cli.FlagExpiration, "invalid"),
				},
				commonFlags...,
			),
			true, 0, nil,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.NewCmdFeeGrant()
			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)

			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.respType), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxWithFeeGrant() {
	clientCtx := s.clientCtx
	granter := s.addedGranter
	granterAddr, err := s.baseCtx.AddressCodec.BytesToString(granter)
	s.Require().NoError(err)

	// creating an account manually (This account won't exist in state)
	k, _, err := s.baseCtx.Keyring.NewMnemonic("grantee", keyring.English, sdk.FullFundraiserPath, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	s.Require().NoError(err)
	pub, err := k.GetPubKey()
	s.Require().NoError(err)
	grantee := sdk.AccAddress(pub.Address())
	granteeAddr, err := s.baseCtx.AddressCodec.BytesToString(grantee)
	s.Require().NoError(err)

	commonFlags := []string{
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
	}

	fee := sdk.NewCoin("stake", sdkmath.NewInt(100))

	args := append(
		[]string{
			granterAddr,
			granteeAddr,
			fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, fee.String()),
			fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
			fmt.Sprintf("--%s=%s", cli.FlagExpiration, getFormattedExpiration(oneYear)),
		},
		commonFlags...,
	)

	cmd := cli.NewCmdFeeGrant()

	var res sdk.TxResponse
	out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, args)
	s.Require().NoError(err)
	s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), &res), out.String())

	testcases := []struct {
		name       string
		from       string
		flags      []string
		expErrCode uint32
	}{
		{
			name:  "granted fee allowance for an account which is not in state and creating any tx with it by using --fee-granter shouldn't fail",
			from:  granteeAddr,
			flags: []string{fmt.Sprintf("--%s=%s", flags.FlagFeeGranter, granterAddr)},
		},
		{
			name:       "--fee-payer should also sign the tx (direct)",
			from:       granteeAddr,
			flags:      []string{fmt.Sprintf("--%s=%s", flags.FlagFeePayer, granterAddr)},
			expErrCode: 4,
		},
		{
			name: "--fee-payer should also sign the tx (amino-json)",
			from: granteeAddr,
			flags: []string{
				fmt.Sprintf("--%s=%s", flags.FlagFeePayer, granterAddr),
				fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
			},
			expErrCode: 4,
		},
		{
			name: "use --fee-payer and --fee-granter together works",
			from: granteeAddr,
			flags: []string{
				fmt.Sprintf("--%s=%s", flags.FlagFeePayer, granteeAddr),
				fmt.Sprintf("--%s=%s", flags.FlagFeeGranter, granterAddr),
			},
		},
	}

	for _, tc := range testcases {
		s.Run(tc.name, func() {
			err := s.msgSubmitLegacyProposal(s.baseCtx, tc.from,
				"Text Proposal", "No desc", "text",
				tc.flags...,
			)
			s.Require().NoError(err)

			var resp sdk.TxResponse
			s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), &resp), out.String())
		})
	}
}

func (s *CLITestSuite) msgSubmitLegacyProposal(clientCtx client.Context, from, title, description, proposalType string, extraArgs ...string) error {
	commonArgs := []string{
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(10))).String()),
	}

	args := append([]string{
		fmt.Sprintf("--%s=%s", govcli.FlagTitle, title),
		fmt.Sprintf("--%s=%s", govcli.FlagDescription, description),   //nolint:staticcheck // SA1019: govcli.FlagDescription is deprecated: use FlagDescription instead
		fmt.Sprintf("--%s=%s", govcli.FlagProposalType, proposalType), //nolint:staticcheck // SA1019: govcli.FlagProposalType is deprecated: use FlagProposalType instead
		fmt.Sprintf("--%s=%s", flags.FlagFrom, from),
	}, commonArgs...)

	args = append(args, extraArgs...)

	cmd := govcli.NewCmdSubmitLegacyProposal()

	out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, args)
	s.Require().NoError(err)
	var resp sdk.TxResponse
	s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), &resp), out.String())

	return err
}

func (s *CLITestSuite) TestFilteredFeeAllowance() {
	granter := s.addedGranter
	granterAddr, err := s.baseCtx.AddressCodec.BytesToString(granter)
	s.Require().NoError(err)
	k, _, err := s.baseCtx.Keyring.NewMnemonic("grantee1", keyring.English, sdk.FullFundraiserPath, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	s.Require().NoError(err)
	pub, err := k.GetPubKey()
	s.Require().NoError(err)
	grantee := sdk.AccAddress(pub.Address())
	granteeAddr, err := s.baseCtx.AddressCodec.BytesToString(grantee)
	s.Require().NoError(err)
	clientCtx := s.clientCtx

	commonFlags := []string{
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))).String()),
	}
	spendLimit := sdk.NewCoin("stake", sdkmath.NewInt(1000))
	allowMsgs := strings.Join([]string{sdk.MsgTypeURL(&v1beta1.MsgSubmitProposal{}), sdk.MsgTypeURL(&v1.MsgVoteWeighted{})}, ",")

	testCases := []struct {
		name         string
		args         []string
		expectErrMsg string
	}{
		{
			"invalid granter address",
			append(
				[]string{
					"not an address",
					"cosmos1nph3cfzk6trsmfxkeu943nvach5qw4vwstnvkl",
					fmt.Sprintf("--%s=%s", cli.FlagAllowedMsgs, allowMsgs),
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, spendLimit.String()),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			"key not found",
		},
		{
			"invalid grantee address",
			append(
				[]string{
					granterAddr,
					"not an address",
					fmt.Sprintf("--%s=%s", cli.FlagAllowedMsgs, allowMsgs),
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, spendLimit.String()),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			"decoding bech32 failed",
		},
		{
			"valid filter fee grant",
			append(
				[]string{
					granterAddr,
					granteeAddr,
					fmt.Sprintf("--%s=%s", cli.FlagAllowedMsgs, allowMsgs),
					fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, spendLimit.String()),
					fmt.Sprintf("--%s=%s", flags.FlagFrom, granter),
				},
				commonFlags...,
			),
			"",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			cmd := cli.NewCmdFeeGrant()
			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			if tc.expectErrMsg != "" {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err)
				msg := &sdk.TxResponse{}
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), msg), out.String())
			}
		})
	}

	// exec filtered fee allowance
	cases := []struct {
		name     string
		malleate func() error
	}{
		{
			"valid proposal tx",
			func() error {
				return s.msgSubmitLegacyProposal(s.baseCtx, granteeAddr,
					"Text Proposal", "No desc", "Text",
					fmt.Sprintf("--%s=%s", flags.FlagFeeGranter, granterAddr),
					fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))).String()),
				)
			},
		},
		{
			"valid weighted_vote tx",
			func() error {
				return s.msgVote(s.baseCtx, granteeAddr, "0", "yes",
					fmt.Sprintf("--%s=%s", flags.FlagFeeGranter, granterAddr),
					fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))).String()),
				)
			},
		},
		{
			"should fail with unauthorized msgs",
			func() error {
				args := append(
					[]string{
						granteeAddr,
						"cosmos14cm33pvnrv2497tyt8sp9yavhmw83nwej3m0e8",
						fmt.Sprintf("--%s=%s", cli.FlagSpendLimit, "100stake"),
						fmt.Sprintf("--%s=%s", flags.FlagFeeGranter, granter),
					},
					commonFlags...,
				)

				cmd := cli.NewCmdFeeGrant()
				out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, args)
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), &sdk.TxResponse{}), out.String())

				return err
			},
		},
	}

	for _, tc := range cases {
		s.Run(tc.name, func() {
			err := tc.malleate()
			s.Require().NoError(err)
		})
	}
}

// msgVote votes for a proposal
func (s *CLITestSuite) msgVote(clientCtx client.Context, from, id, vote string, extraArgs ...string) error {
	commonArgs := []string{
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(10))).String()),
	}
	args := append([]string{
		id,
		vote,
		fmt.Sprintf("--%s=%s", flags.FlagFrom, from),
	}, commonArgs...)

	args = append(args, extraArgs...)
	cmd := govcli.NewCmdWeightedVote()

	out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, args)

	s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), &sdk.TxResponse{}), out.String())

	return err
}

func getFormattedExpiration(duration int64) string {
	return time.Now().Add(time.Duration(duration) * time.Second).Format(time.RFC3339)
}
