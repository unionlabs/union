package cli_test

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	rpcclientmock "github.com/cometbft/cometbft/rpc/client/mock"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	"github.com/cosmos/cosmos-sdk/testutil/network"
	sdk "github.com/cosmos/cosmos-sdk/types"
	testutilmod "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/nft"
	nftmodule "github.com/cosmos/cosmos-sdk/x/nft/module"

	"github.com/cosmos/cosmos-sdk/x/nft/client/cli"
	nfttestutil "github.com/cosmos/cosmos-sdk/x/nft/testutil"
)

const (
	OwnerName  = "owner"
	Owner      = "cosmos1kznrznww4pd6gx0zwrpthjk68fdmqypjpkj5hp"
	OwnerArmor = `-----BEGIN TENDERMINT PRIVATE KEY-----
salt: C3586B75587D2824187D2CDA22B6AFB6
type: secp256k1
kdf: bcrypt

1+15OrCKgjnwym1zO3cjo/SGe3PPqAYChQ5wMHjdUbTZM7mWsH3/ueL6swgjzI3b
DDzEQAPXBQflzNW6wbne9IfT651zCSm+j1MWaGk=
=wEHs
-----END TENDERMINT PRIVATE KEY-----`

	testClassID          = "kitty"
	testClassName        = "Crypto Kitty"
	testClassSymbol      = "kitty"
	testClassDescription = "Crypto Kitty"
	testClassURI         = "class uri"
	testID               = "kitty1"
	testURI              = "kitty uri"
)

var (
	ExpClass = nft.Class{
		Id:          testClassID,
		Name:        testClassName,
		Symbol:      testClassSymbol,
		Description: testClassDescription,
		Uri:         testClassURI,
	}

	ExpNFT = nft.NFT{
		ClassId: testClassID,
		Id:      testID,
		Uri:     testURI,
	}
)

type CLITestSuite struct {
	suite.Suite

	kr        keyring.Keyring
	encCfg    testutilmod.TestEncodingConfig
	baseCtx   client.Context
	clientCtx client.Context
	ctx       context.Context

	owner sdk.AccAddress
}

func TestCLITestSuite(t *testing.T) {
	suite.Run(t, new(CLITestSuite))
}

func (s *CLITestSuite) SetupSuite() {
	s.encCfg = testutilmod.MakeTestEncodingConfig(nftmodule.AppModuleBasic{})
	s.kr = keyring.NewInMemory(s.encCfg.Codec)
	s.baseCtx = client.Context{}.
		WithKeyring(s.kr).
		WithTxConfig(s.encCfg.TxConfig).
		WithCodec(s.encCfg.Codec).
		WithClient(clitestutil.MockTendermintRPC{Client: rpcclientmock.Client{}}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain")

	s.ctx = svrcmd.CreateExecuteContext(context.Background())
	var outBuf bytes.Buffer
	ctxGen := func() client.Context {
		bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
		c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
			Value: bz,
		})
		return s.baseCtx.WithClient(c)
	}
	s.clientCtx = ctxGen().WithOutput(&outBuf)

	cfg, err := network.DefaultConfigWithAppConfig(nfttestutil.AppConfig)
	s.Require().NoError(err)

	genesisState := cfg.GenesisState
	nftGenesis := nft.GenesisState{
		Classes: []*nft.Class{&ExpClass},
		Entries: []*nft.Entry{{
			Owner: Owner,
			Nfts:  []*nft.NFT{&ExpNFT},
		}},
	}

	nftDataBz, err := s.encCfg.Codec.MarshalJSON(&nftGenesis)
	s.Require().NoError(err)
	genesisState[nft.ModuleName] = nftDataBz

	s.initAccount()
}

func (s *CLITestSuite) TestCLITxSend() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	extraArgs := []string{
		fmt.Sprintf("--%s=%s", flags.FlagFrom, OwnerName),
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(10))).String()),
	}

	testCases := []struct {
		name         string
		args         []string
		expectedCode uint32
		expectErr    bool
		expErrMsg    string
	}{
		{
			"class id is empty",
			[]string{
				"",
				testID,
				accounts[0].Address.String(),
			},
			0,
			true,
			"empty class id",
		},
		{
			"nft id is empty",
			[]string{
				testClassID,
				"",
				accounts[0].Address.String(),
			},
			0,
			true,
			"empty nft id",
		},
		{
			"invalid receiver address",
			[]string{
				testClassID,
				testID,
				"invalid receiver",
			},
			0,
			true,
			"Invalid receiver address",
		},
		{
			"valid transaction",
			[]string{
				testClassID,
				testID,
				accounts[0].Address.String(),
			},
			0,
			false,
			"",
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			args := append(tc.args, extraArgs...)
			cmd := cli.NewCmdSend()
			cmd.SetContext(s.ctx)
			cmd.SetArgs(args)

			s.Require().NoError(client.SetCmdClientContextHandler(s.clientCtx, cmd))

			out, err := clitestutil.ExecTestCLICmd(s.clientCtx, cmd, args)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				var txResp sdk.TxResponse
				s.Require().NoError(err)
				s.Require().NoError(s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &txResp), out.String())
				s.Require().Equal(tc.expectedCode, txResp.Code, out.String())
			}
		})
	}
}

func (s *CLITestSuite) initAccount() {
	ctx := s.clientCtx
	err := ctx.Keyring.ImportPrivKey(OwnerName, OwnerArmor, "1234567890")
	s.Require().NoError(err)
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	keyinfo, err := ctx.Keyring.Key(OwnerName)
	s.Require().NoError(err)

	args := []string{
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(10))).String()),
	}

	s.owner, err = keyinfo.GetAddress()
	s.Require().NoError(err)

	amount := sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(200)))
	_, err = clitestutil.MsgSendExec(ctx, accounts[0].Address, s.owner, amount, args...)
	s.Require().NoError(err)
}
