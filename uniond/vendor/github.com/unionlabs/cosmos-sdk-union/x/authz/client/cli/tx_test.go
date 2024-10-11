package cli_test

import (
	"fmt"
	"io"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	_ "cosmossdk.io/api/cosmos/authz/v1beta1"
	govv1 "cosmossdk.io/api/cosmos/gov/v1"
	"cosmossdk.io/core/address"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/authz/client/cli"
	authzclitestutil "cosmossdk.io/x/authz/client/testutil"
	authz "cosmossdk.io/x/authz/module"
	"cosmossdk.io/x/bank"
	banktypes "cosmossdk.io/x/bank/types"

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

var typeMsgVote = sdk.MsgTypeURL(&govv1.MsgVote{})

type CLITestSuite struct {
	suite.Suite

	kr        keyring.Keyring
	encCfg    testutilmod.TestEncodingConfig
	baseCtx   client.Context
	clientCtx client.Context
	grantee   []sdk.AccAddress
	addrs     []sdk.AccAddress

	ac address.Codec
}

func TestCLITestSuite(t *testing.T) {
	suite.Run(t, new(CLITestSuite))
}

func (s *CLITestSuite) SetupSuite() {
	s.encCfg = testutilmod.MakeTestEncodingConfig(codectestutil.CodecOptions{}, bank.AppModule{}, authz.AppModule{})
	s.kr = keyring.NewInMemory(s.encCfg.Codec)

	s.baseCtx = client.Context{}.
		WithKeyring(s.kr).
		WithTxConfig(s.encCfg.TxConfig).
		WithCodec(s.encCfg.Codec).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain").
		WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
		WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
		WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))

	s.ac = addresscodec.NewBech32Codec("cosmos")

	ctxGen := func() client.Context {
		bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
		c := clitestutil.NewMockCometRPCWithResponseQueryValue(bz)
		return s.baseCtx.WithClient(c)
	}
	s.clientCtx = ctxGen()

	val := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)
	s.grantee = make([]sdk.AccAddress, 6)
	valAddr, err := s.baseCtx.AddressCodec.BytesToString(val[0].Address)
	s.Require().NoError(err)

	s.addrs = make([]sdk.AccAddress, 1)
	s.addrs[0] = s.createAccount("validator address")

	// Send some funds to the new account.
	// Create new account in the keyring.
	s.grantee[0] = s.createAccount("grantee1")
	s.msgSendExec(s.grantee[0])

	// Create new account in the keyring.
	s.grantee[1] = s.createAccount("grantee2")
	// Send some funds to the new account.
	s.msgSendExec(s.grantee[1])
	grantee1Addr, err := s.baseCtx.AddressCodec.BytesToString(s.grantee[1])
	s.Require().NoError(err)

	// grant send authorization to grantee2
	out, err := authzclitestutil.CreateGrant(s.clientCtx, []string{
		grantee1Addr,
		"send",
		fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
		fmt.Sprintf("--%s=%s", flags.FlagFrom, valAddr),
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(10))).String()),
		fmt.Sprintf("--%s=%d", cli.FlagExpiration, time.Now().Add(time.Minute*time.Duration(120)).Unix()),
	})
	s.Require().NoError(err)

	var response sdk.TxResponse
	s.Require().NoError(s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &response), out.String())

	// Create new account in the keyring.
	s.grantee[2] = s.createAccount("grantee3")
	grantee2Addr, err := s.baseCtx.AddressCodec.BytesToString(s.grantee[2])
	s.Require().NoError(err)

	// grant send authorization to grantee3
	_, err = authzclitestutil.CreateGrant(s.clientCtx, []string{
		grantee2Addr,
		"send",
		fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
		fmt.Sprintf("--%s=%s", flags.FlagFrom, valAddr),
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(10))).String()),
		fmt.Sprintf("--%s=%d", cli.FlagExpiration, time.Now().Add(time.Minute*time.Duration(120)).Unix()),
	})
	s.Require().NoError(err)

	// Create new accounts in the keyring.
	s.grantee[3] = s.createAccount("grantee4")
	s.msgSendExec(s.grantee[3])
	grantee3Addr, err := s.baseCtx.AddressCodec.BytesToString(s.grantee[3])
	s.Require().NoError(err)

	s.grantee[4] = s.createAccount("grantee5")
	s.grantee[5] = s.createAccount("grantee6")

	// grant send authorization with allow list to grantee4
	out, err = authzclitestutil.CreateGrant(s.clientCtx,
		[]string{
			grantee3Addr,
			"send",
			fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
			fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
			fmt.Sprintf("--%s=%s", flags.FlagFrom, valAddr),
			fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
			fmt.Sprintf("--%s=%d", cli.FlagExpiration, time.Now().Add(time.Minute*time.Duration(120)).Unix()),
			fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
			fmt.Sprintf("--%s=%s", cli.FlagAllowList, s.grantee[4]),
		},
	)
	s.Require().NoError(err)
	s.Require().NoError(s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &response), out.String())
}

func (s *CLITestSuite) createAccount(uid string) sdk.AccAddress {
	// Create new account in the keyring.
	k, _, err := s.clientCtx.Keyring.NewMnemonic(uid, keyring.English, sdk.FullFundraiserPath, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	s.Require().NoError(err)

	addr, err := k.GetAddress()
	s.Require().NoError(err)

	return addr
}

func (s *CLITestSuite) msgSendExec(grantee sdk.AccAddress) {
	val := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)
	// Send some funds to the new account.
	_, err := s.ac.StringToBytes("cosmos16zex22087zs656t0vedytv5wqhm6axxd5679ry")
	s.Require().NoError(err)

	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(200)))
	from := val[0].Address
	fromAddr, err := s.clientCtx.AddressCodec.BytesToString(from)
	s.Require().NoError(err)
	granteeAddr, err := s.clientCtx.AddressCodec.BytesToString(grantee)
	s.Require().NoError(err)
	msgSend := &banktypes.MsgSend{
		FromAddress: fromAddr,
		ToAddress:   granteeAddr,
		Amount:      coins,
	}

	_, err = clitestutil.SubmitTestTx(s.clientCtx, msgSend, from, clitestutil.TestTxConfig{})
	s.Require().NoError(err)
}

func (s *CLITestSuite) TestCLITxGrantAuthorization() {
	val := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)
	valAddress, err := s.clientCtx.ValidatorAddressCodec.BytesToString(s.addrs[0])
	s.Require().NoError(err)
	fromAddr, err := s.baseCtx.AddressCodec.BytesToString(val[0].Address)
	s.Require().NoError(err)

	grantee := s.grantee[0]
	granteeAddr, err := s.baseCtx.AddressCodec.BytesToString(grantee)
	s.Require().NoError(err)

	twoHours := time.Now().Add(time.Minute * 120).Unix()
	pastHour := time.Now().Add(-time.Minute * 60).Unix()

	testCases := []struct {
		name      string
		args      []string
		expectErr bool
		expErrMsg string
	}{
		{
			"Invalid granter Address",
			[]string{
				"grantee_addr",
				"send",
				fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, "granter"),
				fmt.Sprintf("--%s=true", flags.FlagGenerateOnly),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
			},
			true,
			"key not found",
		},
		{
			"Invalid grantee Address",
			[]string{
				"grantee_addr",
				"send",
				fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=true", flags.FlagGenerateOnly),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
			},
			true,
			"invalid separator index",
		},
		{
			"Invalid spend limit",
			[]string{
				granteeAddr,
				"send",
				fmt.Sprintf("--%s=0stake", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=true", flags.FlagGenerateOnly),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
			},
			true,
			"spend-limit should be greater than zero",
		},
		{
			"Invalid expiration time",
			[]string{
				granteeAddr,
				"send",
				fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=true", flags.FlagBroadcastMode),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, pastHour),
			},
			true,
			"",
		},
		{
			"fail with error invalid msg-type",
			[]string{
				granteeAddr,
				"generic",
				fmt.Sprintf("--%s=invalid-msg-type", cli.FlagMsgType),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
			},
			false,
			"",
		},
		{
			"invalid bond denom for tx delegate authorization allowed validators",
			[]string{
				granteeAddr,
				"delegate",
				fmt.Sprintf("--%s=100xyz", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=%s", cli.FlagAllowedValidators, valAddress),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
			},
			true,
			"invalid denom",
		},
		{
			"invalid bond denom for tx delegate authorization deny validators",
			[]string{
				granteeAddr,
				"delegate",
				fmt.Sprintf("--%s=100xyz", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=%s", cli.FlagDenyValidators, valAddress),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
			},
			true,
			"invalid denom",
		},
		{
			"invalid bond denom for tx undelegate authorization",
			[]string{
				granteeAddr,
				"unbond",
				fmt.Sprintf("--%s=100xyz", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=%s", cli.FlagAllowedValidators, valAddress),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
			},
			true,
			"invalid denom",
		},
		{
			"invalid bond denom for tx redelegate authorization",
			[]string{
				granteeAddr,
				"redelegate",
				fmt.Sprintf("--%s=100xyz", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=%s", cli.FlagAllowedValidators, valAddress),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
			},
			true,
			"invalid denom",
		},
		{
			"invalid decimal coin expression with more than single coin",
			[]string{
				granteeAddr,
				"delegate",
				fmt.Sprintf("--%s=100stake,20xyz", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=%s", cli.FlagAllowedValidators, valAddress),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
			},
			true,
			"invalid character in denomination",
		},
		{
			"invalid authorization type",
			[]string{
				granteeAddr,
				"invalid authz type",
			},
			true,
			"invalid authorization type",
		},
		{
			"Valid tx send authorization",
			[]string{
				granteeAddr,
				"send",
				fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
			},
			false,
			"",
		},
		{
			"Valid tx send authorization with allow list",
			[]string{
				granteeAddr,
				"send",
				fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
				fmt.Sprintf("--%s=%s", cli.FlagAllowList, s.grantee[1]),
			},
			false,
			"",
		},
		{
			"Invalid tx send authorization with duplicate allow list",
			[]string{
				granteeAddr,
				"send",
				fmt.Sprintf("--%s=100stake", cli.FlagSpendLimit),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
				fmt.Sprintf("--%s=%s", cli.FlagAllowList, fmt.Sprintf("%s,%s", s.grantee[1], s.grantee[1])),
			},
			true,
			"duplicate address",
		},
		{
			"Valid tx generic authorization",
			[]string{
				granteeAddr,
				"generic",
				fmt.Sprintf("--%s=%s", cli.FlagMsgType, typeMsgVote),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
			},
			false,
			"",
		},
		{
			"fail when granter = grantee",
			[]string{
				granteeAddr,
				"generic",
				fmt.Sprintf("--%s=%s", cli.FlagMsgType, typeMsgVote),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, granteeAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
			},
			true,
			"grantee and granter should be different",
		},
		{
			"Valid tx with amino",
			[]string{
				granteeAddr,
				"generic",
				fmt.Sprintf("--%s=%s", cli.FlagMsgType, typeMsgVote),
				fmt.Sprintf("--%s=%s", flags.FlagFrom, fromAddr),
				fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
				fmt.Sprintf("--%s=%d", cli.FlagExpiration, twoHours),
				fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
				fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
				fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
			},
			false,
			"",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			out, err := authzclitestutil.CreateGrant(s.clientCtx,
				tc.args,
			)
			if tc.expectErr {
				s.Require().Error(err, out)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				var txResp sdk.TxResponse
				s.Require().NoError(err)
				s.Require().NoError(s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &txResp), out.String())
			}
		})
	}
}
