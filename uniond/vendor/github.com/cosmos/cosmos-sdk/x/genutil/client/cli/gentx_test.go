package cli_test

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"path/filepath"
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	rpcclientmock "github.com/cometbft/cometbft/rpc/client/mock"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	testutilmod "github.com/cosmos/cosmos-sdk/types/module/testutil"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	"github.com/cosmos/cosmos-sdk/x/genutil/client/cli"
	stakingcli "github.com/cosmos/cosmos-sdk/x/staking/client/cli"
)

type CLITestSuite struct {
	suite.Suite

	kr        keyring.Keyring
	encCfg    testutilmod.TestEncodingConfig
	baseCtx   client.Context
	clientCtx client.Context
}

func TestCLITestSuite(t *testing.T) {
	suite.Run(t, new(CLITestSuite))
}

func (s *CLITestSuite) SetupSuite() {
	s.encCfg = testutilmod.MakeTestEncodingConfig(genutil.AppModuleBasic{})
	s.kr = keyring.NewInMemory(s.encCfg.Codec)
	s.baseCtx = client.Context{}.
		WithKeyring(s.kr).
		WithTxConfig(s.encCfg.TxConfig).
		WithCodec(s.encCfg.Codec).
		WithClient(clitestutil.MockTendermintRPC{Client: rpcclientmock.Client{}}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain")

	var outBuf bytes.Buffer
	ctxGen := func() client.Context {
		bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
		c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
			Value: bz,
		})
		return s.baseCtx.WithClient(c)
	}
	s.clientCtx = ctxGen().WithOutput(&outBuf)
}

func (s *CLITestSuite) TestGenTxCmd() {
	amount := sdk.NewCoin("stake", sdk.NewInt(12))

	tests := []struct {
		name         string
		args         []string
		expCmdOutput string
	}{
		{
			name: "invalid commission rate returns error",
			args: []string{
				fmt.Sprintf("--%s=%s", flags.FlagChainID, s.baseCtx.ChainID),
				fmt.Sprintf("--%s=1", stakingcli.FlagCommissionRate),
				"node0",
				amount.String(),
			},
			expCmdOutput: fmt.Sprintf("--%s=%s --%s=1 %s %s", flags.FlagChainID, s.baseCtx.ChainID, stakingcli.FlagCommissionRate, "node0", amount.String()),
		},
		{
			name: "valid gentx",
			args: []string{
				fmt.Sprintf("--%s=%s", flags.FlagChainID, s.baseCtx.ChainID),
				"node0",
				amount.String(),
			},
			expCmdOutput: fmt.Sprintf("--%s=%s %s %s", flags.FlagChainID, s.baseCtx.ChainID, "node0", amount.String()),
		},
		{
			name: "invalid pubkey",
			args: []string{
				fmt.Sprintf("--%s=%s", flags.FlagChainID, "test-chain-1"),
				fmt.Sprintf("--%s={\"key\":\"BOIkjkFruMpfOFC9oNPhiJGfmY2pHF/gwHdLDLnrnS0=\"}", stakingcli.FlagPubKey),
				"node0",
				amount.String(),
			},
			expCmdOutput: fmt.Sprintf("--%s=test-chain-1 --%s={\"key\":\"BOIkjkFruMpfOFC9oNPhiJGfmY2pHF/gwHdLDLnrnS0=\"} %s %s ", flags.FlagChainID, stakingcli.FlagPubKey, "node0", amount.String()),
		},
		{
			name: "valid pubkey flag",
			args: []string{
				fmt.Sprintf("--%s=%s", flags.FlagChainID, "test-chain-1"),
				fmt.Sprintf("--%s={\"@type\":\"/cosmos.crypto.ed25519.PubKey\",\"key\":\"BOIkjkFruMpfOFC9oNPhiJGfmY2pHF/gwHdLDLnrnS0=\"}", stakingcli.FlagPubKey),
				"node0",
				amount.String(),
			},
			expCmdOutput: fmt.Sprintf("--%s=test-chain-1 --%s={\"@type\":\"/cosmos.crypto.ed25519.PubKey\",\"key\":\"BOIkjkFruMpfOFC9oNPhiJGfmY2pHF/gwHdLDLnrnS0=\"} %s %s ", flags.FlagChainID, stakingcli.FlagPubKey, "node0", amount.String()),
		},
	}

	for _, tc := range tests {
		tc := tc

		dir := s.T().TempDir()
		genTxFile := filepath.Join(dir, "myTx")
		tc.args = append(tc.args, fmt.Sprintf("--%s=%s", flags.FlagOutputDocument, genTxFile))

		s.Run(tc.name, func() {
			clientCtx := s.clientCtx
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd := cli.GenTxCmd(
				module.NewBasicManager(),
				clientCtx.TxConfig,
				banktypes.GenesisBalancesIterator{},
				clientCtx.HomeDir,
			)
			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(clientCtx, cmd))

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}
		})
	}
}
