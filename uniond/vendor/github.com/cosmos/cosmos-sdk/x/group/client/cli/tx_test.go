package cli_test

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"strconv"
	"strings"
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	rpcclientmock "github.com/cometbft/cometbft/rpc/client/mock"
	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	sdk "github.com/cosmos/cosmos-sdk/types"
	testutilmod "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/group"
	groupcli "github.com/cosmos/cosmos-sdk/x/group/client/cli"
	groupmodule "github.com/cosmos/cosmos-sdk/x/group/module"
)

const validMetadata = "metadata"

var tooLongMetadata = strings.Repeat("A", 256)

type CLITestSuite struct {
	suite.Suite

	kr          keyring.Keyring
	encCfg      testutilmod.TestEncodingConfig
	baseCtx     client.Context
	clientCtx   client.Context
	group       *group.GroupInfo
	commonFlags []string
}

func TestCLITestSuite(t *testing.T) {
	suite.Run(t, new(CLITestSuite))
}

func (s *CLITestSuite) SetupSuite() {
	s.encCfg = testutilmod.MakeTestEncodingConfig(groupmodule.AppModuleBasic{})
	s.kr = keyring.NewInMemory(s.encCfg.Codec)
	s.baseCtx = client.Context{}.
		WithKeyring(s.kr).
		WithTxConfig(s.encCfg.TxConfig).
		WithCodec(s.encCfg.Codec).
		WithClient(clitestutil.MockTendermintRPC{Client: rpcclientmock.Client{}}).
		WithAccountRetriever(client.MockAccountRetriever{}).
		WithOutput(io.Discard).
		WithChainID("test-chain")

	s.commonFlags = []string{
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(10))).String()),
	}

	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)
	val := accounts[0]

	var outBuf bytes.Buffer
	ctxGen := func() client.Context {
		bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
		c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
			Value: bz,
		})
		return s.baseCtx.WithClient(c)
	}
	s.clientCtx = ctxGen().WithOutput(&outBuf)

	// create a new account
	info, _, err := s.clientCtx.Keyring.NewMnemonic("NewValidator", keyring.English, sdk.FullFundraiserPath, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	s.Require().NoError(err)

	pk, err := info.GetPubKey()
	s.Require().NoError(err)

	account := sdk.AccAddress(pk.Address())
	_, err = clitestutil.MsgSendExec(
		s.clientCtx,
		val.Address,
		account,
		sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(2000))), fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(10))).String()),
	)
	s.Require().NoError(err)

	memberWeight := "3"
	// create a group
	validMembers := fmt.Sprintf(`
	{
		"members": [
			{
				"address": "%s",
				"weight": "%s",
				"metadata": "%s"
			}
		]
	}`, val.Address.String(), memberWeight, validMetadata)
	validMembersFile := testutil.WriteToNewTempFile(s.T(), validMembers)
	out, err := clitestutil.ExecTestCLICmd(s.clientCtx, groupcli.MsgCreateGroupCmd(),
		append(
			[]string{
				val.Address.String(),
				validMetadata,
				validMembersFile.Name(),
			},
			s.commonFlags...,
		),
	)

	s.Require().NoError(err, out.String())
	txResp := sdk.TxResponse{}
	s.Require().NoError(s.clientCtx.Codec.UnmarshalJSON(out.Bytes(), &txResp), out.String())
	s.Require().Equal(uint32(0), txResp.Code, out.String())

	s.group = &group.GroupInfo{Id: 1, Admin: val.Address.String(), Metadata: validMetadata, TotalWeight: "3", Version: 1}
}

func (s *CLITestSuite) TestTxCreateGroup() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	cmd := groupcli.MsgCreateGroupCmd()
	cmd.SetOutput(io.Discard)

	validMembers := fmt.Sprintf(`{"members": [{
		"address": "%s",
		  "weight": "1",
		  "metadata": "%s"
	  }]}`, accounts[0].Address.String(), validMetadata)
	validMembersFile := testutil.WriteToNewTempFile(s.T(), validMembers)

	invalidMembersAddress := `{"members": [{
		  "address": "",
		  "weight": "1"
	  }]}`
	invalidMembersAddressFile := testutil.WriteToNewTempFile(s.T(), invalidMembersAddress)

	invalidMembersWeight := fmt.Sprintf(`{"members": [{
			"address": "%s",
			  "weight": "0"
		  }]}`, accounts[0].Address.String())
	invalidMembersWeightFile := testutil.WriteToNewTempFile(s.T(), invalidMembersWeight)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		respType     proto.Message
		expCmdOutput string
		expectErr    bool
		expectErrMsg string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					"",
					validMembersFile.Name(),
				},
				s.commonFlags...,
			),
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), "", validMembersFile.Name()),
			false,
			"",
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					"",
					validMembersFile.Name(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), "", validMembersFile.Name()),
			false,
			"",
		},
		{
			"invalid members address",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					"null",
					invalidMembersAddressFile.Name(),
				},
				s.commonFlags...,
			),
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), "null", invalidMembersAddressFile.Name()),
			true,
			"message validation failed: address: empty address string is not allowed",
		},
		{
			"invalid members weight",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					"null",
					invalidMembersWeightFile.Name(),
				},
				s.commonFlags...,
			),
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), "null", invalidMembersWeightFile.Name()),
			true,
			"expected a positive decimal, got 0: invalid decimal string",
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err)
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.respType), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxUpdateGroupAdmin() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 2)

	cmd := groupcli.MsgUpdateGroupAdminCmd()
	cmd.SetOutput(io.Discard)

	var outBuf bytes.Buffer

	ctxGen := func() client.Context {
		bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
		c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
			Value: bz,
		})
		return s.baseCtx.WithClient(c)
	}
	clientCtx := ctxGen().WithOutput(&outBuf)
	ctx := svrcmd.CreateExecuteContext(context.Background())

	cmd.SetContext(ctx)

	s.Require().NoError(client.SetCmdClientContextHandler(clientCtx, cmd))

	groupIDs := make([]string, 2)
	for i := 0; i < 2; i++ {
		validMembers := fmt.Sprintf(`{"members": [{
	  "address": "%s",
		"weight": "1",
		"metadata": "%s"
	}]}`, accounts[0].Address.String(), validMetadata)
		validMembersFile := testutil.WriteToNewTempFile(s.T(), validMembers)
		out, err := clitestutil.ExecTestCLICmd(clientCtx, groupcli.MsgCreateGroupCmd(),
			append(
				[]string{
					accounts[0].Address.String(),
					validMetadata,
					validMembersFile.Name(),
				},
				s.commonFlags...,
			),
		)
		s.Require().NoError(err, out.String())
		groupIDs[i] = fmt.Sprintf("%d", i+1)
	}

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		respType     proto.Message
		expCmdOutput string
		expectErr    bool
		expectErrMsg string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					groupIDs[0],
					accounts[1].Address.String(),
				},
				s.commonFlags...,
			),
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), groupIDs[0], accounts[1].Address.String()),
			false,
			"",
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					groupIDs[1],
					accounts[1].Address.String(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s --%s=%s", accounts[0].Address.String(), groupIDs[1], accounts[1].Address.String(), flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
			false,
			"",
		},
		{
			"group id invalid",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					"",
					accounts[1].Address.String(),
				},
				s.commonFlags...,
			),
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), "", accounts[1].Address.String()),
			true,
			"strconv.ParseUint: parsing \"\": invalid syntax",
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err)
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.respType), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxUpdateGroupMetadata() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	cmd := groupcli.MsgUpdateGroupMetadataCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					"1",
					validMetadata,
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), "1", validMetadata),
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					"1",
					validMetadata,
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s --%s=%s", accounts[0].Address.String(), "1", validMetadata, flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"group metadata too long",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					strconv.FormatUint(s.group.Id, 10),
					strings.Repeat("a", 256),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), strconv.FormatUint(s.group.Id, 10), strings.Repeat("a", 256)),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}
		})
	}
}

func (s *CLITestSuite) TestTxUpdateGroupMembers() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 3)
	groupPolicyAddress := accounts[2]

	cmd := groupcli.MsgUpdateGroupMembersCmd()
	cmd.SetOutput(io.Discard)

	groupID := "1"

	validUpdatedMembersFileName := testutil.WriteToNewTempFile(s.T(), fmt.Sprintf(`{"members": [{
		"address": "%s",
		"weight": "0",
		"metadata": "%s"
	}, {
		"address": "%s",
		"weight": "1",
		"metadata": "%s"
	}]}`, accounts[1], validMetadata, groupPolicyAddress, validMetadata)).Name()

	invalidMembersMetadata := fmt.Sprintf(`{"members": [{
		"address": "%s",
		  "weight": "1",
		  "metadata": "%s"
	  }]}`, accounts[1], tooLongMetadata)
	invalidMembersMetadataFileName := testutil.WriteToNewTempFile(s.T(), invalidMembersMetadata).Name()

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					groupID,
					validUpdatedMembersFileName,
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), groupID, validUpdatedMembersFileName),
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					groupID,
					validUpdatedMembersFileName,
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s --%s=%s", accounts[0].Address.String(), groupID, validUpdatedMembersFileName, flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"group member metadata too long",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					groupID,
					invalidMembersMetadataFileName,
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), groupID, invalidMembersMetadataFileName),
		},
		{
			"group doesn't exist",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					"12345",
					validUpdatedMembersFileName,
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", accounts[0].Address.String(), "12345", validUpdatedMembersFileName),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}
		})
	}
}

func (s *CLITestSuite) TestTxCreateGroupWithPolicy() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	cmd := groupcli.MsgCreateGroupWithPolicyCmd()
	cmd.SetOutput(io.Discard)

	validMembers := fmt.Sprintf(`{"members": [{
		"address": "%s",
		  "weight": "1",
		  "metadata": "%s"
	}]}`, accounts[0].Address.String(), validMetadata)
	validMembersFile := testutil.WriteToNewTempFile(s.T(), validMembers)

	invalidMembersAddress := `{"members": [{
	  "address": "",
	  "weight": "1"
	}]}`
	invalidMembersAddressFile := testutil.WriteToNewTempFile(s.T(), invalidMembersAddress)

	invalidMembersWeight := fmt.Sprintf(`{"members": [{
		"address": "%s",
		  "weight": "0"
	}]}`, accounts[0].Address.String())
	invalidMembersWeightFile := testutil.WriteToNewTempFile(s.T(), invalidMembersWeight)

	thresholdDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type": "/cosmos.group.v1.ThresholdDecisionPolicy","threshold": "1","windows": {"voting_period":"1s"}}`)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expectErr    bool
		expectErrMsg string
		respType     proto.Message
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					validMetadata,
					validMetadata,
					validMembersFile.Name(),
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%v", groupcli.FlagGroupPolicyAsAdmin, false),
				},
				s.commonFlags...,
			),
			false,
			"",
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s %s %s --%s=%v", accounts[0].Address.String(), validMetadata, validMetadata, validMembersFile.Name(), thresholdDecisionPolicyFile.Name(), groupcli.FlagGroupPolicyAsAdmin, false),
		},
		{
			"group-policy-as-admin is true",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					validMetadata,
					validMetadata,
					validMembersFile.Name(),
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%v", groupcli.FlagGroupPolicyAsAdmin, true),
				},
				s.commonFlags...,
			),
			false,
			"",
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s %s %s --%s=%v", accounts[0].Address.String(), validMetadata, validMetadata, validMembersFile.Name(), thresholdDecisionPolicyFile.Name(), groupcli.FlagGroupPolicyAsAdmin, true),
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					validMetadata,
					validMetadata,
					validMembersFile.Name(),
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%v", groupcli.FlagGroupPolicyAsAdmin, false),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			false,
			"",
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s %s %s --%s=%v --%s=%s", accounts[0].Address.String(), validMetadata, validMetadata, validMembersFile.Name(), thresholdDecisionPolicyFile.Name(), groupcli.FlagGroupPolicyAsAdmin, false, flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"invalid members address",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					validMetadata,
					validMetadata,
					invalidMembersAddressFile.Name(),
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%v", groupcli.FlagGroupPolicyAsAdmin, false),
				},
				s.commonFlags...,
			),
			true,
			"message validation failed: address: empty address string is not allowed",
			nil,
			fmt.Sprintf("%s %s %s %s %s --%s=%v", accounts[0].Address.String(), validMetadata, validMetadata, invalidMembersAddressFile.Name(), thresholdDecisionPolicyFile.Name(), groupcli.FlagGroupPolicyAsAdmin, false),
		},
		{
			"invalid members weight",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					accounts[0].Address.String(),
					validMetadata,
					validMetadata,
					invalidMembersWeightFile.Name(),
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%v", groupcli.FlagGroupPolicyAsAdmin, false),
				},
				s.commonFlags...,
			),
			true,
			"expected a positive decimal, got 0: invalid decimal string",
			nil,
			fmt.Sprintf("%s %s %s %s %s --%s=%v", accounts[0].Address.String(), validMetadata, validMetadata, invalidMembersWeightFile.Name(), thresholdDecisionPolicyFile.Name(), groupcli.FlagGroupPolicyAsAdmin, false),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err, out.String())
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.respType), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxCreateGroupPolicy() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 2)
	val := accounts[0]

	groupID := s.group.Id

	thresholdDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type": "/cosmos.group.v1.ThresholdDecisionPolicy","threshold": "1","windows": {"voting_period":"1s"}}`)

	percentageDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"0.5", "windows":{"voting_period":"1s"}}`)
	invalidNegativePercentageDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"-0.5", "windows":{"voting_period":"1s"}}`)
	invalidPercentageDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"2", "windows":{"voting_period":"1s"}}`)

	cmd := groupcli.MsgCreateGroupPolicyCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expectErr    bool
		expectErrMsg string
		respType     proto.Message
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					val.Address.String(),
					fmt.Sprintf("%v", groupID),
					validMetadata,
					thresholdDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			false,
			"",
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s %s", val.Address.String(), fmt.Sprintf("%v", groupID), validMetadata, thresholdDecisionPolicyFile.Name()),
		},
		{
			"correct data with percentage decision policy",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					val.Address.String(),
					fmt.Sprintf("%v", groupID),
					validMetadata,
					percentageDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			false,
			"",
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s %s", val.Address.String(), fmt.Sprintf("%v", groupID), validMetadata, percentageDecisionPolicyFile.Name()),
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					val.Address.String(),
					fmt.Sprintf("%v", groupID),
					validMetadata,
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			false,
			"",
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s %s --%s=%s", val.Address.String(), fmt.Sprintf("%v", groupID), validMetadata, thresholdDecisionPolicyFile.Name(), flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"wrong admin",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					"wrongAdmin",
					fmt.Sprintf("%v", groupID),
					validMetadata,
					thresholdDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			true,
			"key not found",
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s %s", "wrongAdmin", fmt.Sprintf("%v", groupID), validMetadata, thresholdDecisionPolicyFile.Name()),
		},
		{
			"invalid percentage decision policy with negative value",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					val.Address.String(),
					fmt.Sprintf("%v", groupID),
					validMetadata,
					invalidNegativePercentageDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			true,
			"expected a positive decimal",
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s %s", val.Address.String(), fmt.Sprintf("%v", groupID), validMetadata, invalidNegativePercentageDecisionPolicyFile.Name()),
		},
		{
			"invalid percentage decision policy with value greater than 1",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					val.Address.String(),
					fmt.Sprintf("%v", groupID),
					validMetadata,
					invalidPercentageDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			true,
			"percentage must be > 0 and <= 1",
			&sdk.TxResponse{},
			fmt.Sprintf("%s %s %s %s", val.Address.String(), fmt.Sprintf("%v", groupID), validMetadata, invalidPercentageDecisionPolicyFile.Name()),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(clientCtx, cmd, tc.args)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err, out.String())
				s.Require().NoError(clientCtx.Codec.UnmarshalJSON(out.Bytes(), tc.respType), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxUpdateGroupPolicyAdmin() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 4)
	newAdmin := accounts[0]
	groupPolicyAdmin := accounts[1]
	groupPolicyAddress := accounts[2]

	commonFlags := s.commonFlags
	commonFlags = append(commonFlags, fmt.Sprintf("--%s=%d", flags.FlagGas, 300000))

	cmd := groupcli.MsgUpdateGroupPolicyAdminCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.Address.String(),
					groupPolicyAddress.Address.String(),
					newAdmin.Address.String(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.Address.String(), groupPolicyAddress.Address.String(), newAdmin.Address.String()),
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.Address.String(),
					groupPolicyAddress.Address.String(),
					newAdmin.Address.String(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s --%s=%s", groupPolicyAdmin.Address.String(), groupPolicyAddress.Address.String(), newAdmin.Address.String(), flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"wrong admin",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					"wrong admin",
					groupPolicyAddress.Address.String(),
					newAdmin.Address.String(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", "wrong admin", groupPolicyAddress.Address.String(), newAdmin.Address.String()),
		},
		{
			"wrong group policy",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.Address.String(),
					"wrong group policy",
					newAdmin.Address.String(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.Address.String(), "wrong group policy", newAdmin.Address.String()),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}
		})
	}
}

func (s *CLITestSuite) TestTxUpdateGroupPolicyDecisionPolicy() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 3)
	newAdmin := accounts[0]
	groupPolicyAdmin := accounts[1]
	groupPolicyAddress := accounts[2]

	commonFlags := s.commonFlags
	commonFlags = append(commonFlags, fmt.Sprintf("--%s=%d", flags.FlagGas, 300000))

	thresholdDecisionPolicy := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.ThresholdDecisionPolicy", "threshold":"1", "windows":{"voting_period":"40000s"}}`)
	percentageDecisionPolicy := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"0.5", "windows":{"voting_period":"40000s"}}`)
	invalidNegativePercentageDecisionPolicy := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"-0.5", "windows":{"voting_period":"1s"}}`)
	invalidPercentageDecisionPolicy := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"2", "windows":{"voting_period":"40000s"}}`)

	cmd := groupcli.MsgUpdateGroupPolicyDecisionPolicyCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.Address.String(),
					groupPolicyAddress.Address.String(),
					thresholdDecisionPolicy.Name(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.Address.String(), groupPolicyAddress.Address.String(), thresholdDecisionPolicy.Name()),
		},
		{
			"correct data with percentage decision policy",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.Address.String(),
					groupPolicyAddress.Address.String(),
					percentageDecisionPolicy.Name(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.Address.String(), groupPolicyAddress.Address.String(), percentageDecisionPolicy.Name()),
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.Address.String(),
					groupPolicyAddress.Address.String(),
					thresholdDecisionPolicy.Name(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s --%s=%s", groupPolicyAdmin.Address.String(), groupPolicyAddress.Address.String(), thresholdDecisionPolicy.Name(), flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"wrong admin",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					newAdmin.Address.String(),
					groupPolicyAddress.Address.String(),
					thresholdDecisionPolicy.Name(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", newAdmin.Address.String(), groupPolicyAddress.Address.String(), thresholdDecisionPolicy.Name()),
		},
		{
			"wrong group policy",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.Address.String(),
					"wrong group policy",
					thresholdDecisionPolicy.Name(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.Address.String(), "wrong group policy", thresholdDecisionPolicy.Name()),
		},
		{
			"invalid percentage decision policy with negative value",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.Address.String(),
					groupPolicyAddress.Address.String(),
					invalidNegativePercentageDecisionPolicy.Name(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.Address.String(), groupPolicyAddress.Address.String(), invalidNegativePercentageDecisionPolicy.Name()),
		},
		{
			"invalid percentage decision policy with value greater than 1",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.Address.String(),
					groupPolicyAddress.Address.String(),
					invalidPercentageDecisionPolicy.Name(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.Address.String(), groupPolicyAddress.Address.String(), invalidPercentageDecisionPolicy.Name()),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}
		})
	}
}

func (s *CLITestSuite) TestTxUpdateGroupPolicyMetadata() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 2)
	groupPolicyAdmin := accounts[0].Address
	groupPolicyAddress := accounts[1].Address

	commonFlags := s.commonFlags
	commonFlags = append(commonFlags, fmt.Sprintf("--%s=%d", flags.FlagGas, 300000))

	cmd := groupcli.MsgUpdateGroupPolicyMetadataCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.String(),
					groupPolicyAddress.String(),
					validMetadata,
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.String(), groupPolicyAddress.String(), validMetadata),
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.String(),
					groupPolicyAddress.String(),
					validMetadata,
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s --%s=%s", groupPolicyAdmin.String(), groupPolicyAddress.String(), validMetadata, flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"long metadata",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.String(),
					groupPolicyAddress.String(),
					strings.Repeat("a", 500),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.String(), groupPolicyAddress.String(), strings.Repeat("a", 500)),
		},
		{
			"wrong admin",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					"wrong admin",
					groupPolicyAddress.String(),
					validMetadata,
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", "wrong admin", groupPolicyAddress.String(), validMetadata),
		},
		{
			"wrong group policy",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					groupPolicyAdmin.String(),
					"wrong group policy",
					validMetadata,
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin.String(), "wrong group policy", validMetadata),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}
		})
	}
}

func (s *CLITestSuite) TestTxSubmitProposal() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 2)
	groupPolicyAddress := accounts[1].Address

	p := groupcli.Proposal{
		GroupPolicyAddress: groupPolicyAddress.String(),
		Messages:           []json.RawMessage{},
		Metadata:           validMetadata,
		Proposers:          []string{accounts[0].Address.String()},
	}
	bz, err := json.Marshal(&p)
	s.Require().NoError(err)
	proposalFile := testutil.WriteToNewTempFile(s.T(), string(bz))

	cmd := groupcli.MsgSubmitProposalCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					proposalFile.Name(),
				},
				s.commonFlags...,
			),
			proposalFile.Name(),
		},
		{
			"with try exec",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					proposalFile.Name(),
					fmt.Sprintf("--%s=try", groupcli.FlagExec),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s --%s=try", proposalFile.Name(), groupcli.FlagExec),
		},
		{
			"with try exec, not enough yes votes for proposal to pass",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					proposalFile.Name(),
					fmt.Sprintf("--%s=try", groupcli.FlagExec),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s --%s=try", proposalFile.Name(), groupcli.FlagExec),
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					proposalFile.Name(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s --%s=%s", proposalFile.Name(), flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}
		})
	}
}

func (s *CLITestSuite) TestTxVote() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 4)

	cmd := groupcli.MsgVoteCmd()
	cmd.SetOutput(io.Discard)

	ids := make([]string, 4)
	for i := 0; i < len(ids); i++ {
		ids[i] = fmt.Sprint(i + 1)
	}

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					ids[0],
					accounts[0].Address.String(),
					"VOTE_OPTION_YES",
					"",
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", ids[0], accounts[0].Address.String(), "VOTE_OPTION_YES"),
		},
		{
			"with try exec",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					ids[1],
					accounts[0].Address.String(),
					"VOTE_OPTION_YES",
					"",
					fmt.Sprintf("--%s=try", groupcli.FlagExec),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s %s --%s=try", ids[1], accounts[0].Address.String(), "VOTE_OPTION_YES", "", groupcli.FlagExec),
		},
		{
			"with amino-json",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					ids[3],
					accounts[0].Address.String(),
					"VOTE_OPTION_YES",
					"",
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s %s --%s=%s", ids[3], accounts[0].Address.String(), "VOTE_OPTION_YES", "", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"metadata too long",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					ids[2],
					accounts[0].Address.String(),
					"VOTE_OPTION_YES",
					tooLongMetadata,
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s %s", ids[2], accounts[0].Address.String(), "VOTE_OPTION_YES", tooLongMetadata),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}
		})
	}
}

func (s *CLITestSuite) TestTxWithdrawProposal() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	cmd := groupcli.MsgWithdrawProposalCmd()
	cmd.SetOutput(io.Discard)

	ids := make([]string, 2)
	ids[0] = "1"
	ids[1] = "2"

	testCases := []struct {
		name         string
		ctxGen       func() client.Context
		args         []string
		expCmdOutput string
	}{
		{
			"correct data",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					ids[0],
					accounts[0].Address.String(),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s", ids[0], accounts[0].Address.String()),
		},
		{
			"wrong admin",
			func() client.Context {
				bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
				c := clitestutil.NewMockTendermintRPC(abci.ResponseQuery{
					Value: bz,
				})
				return s.baseCtx.WithClient(c)
			},
			append(
				[]string{
					ids[1],
					"wrongAdmin",
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s", ids[1], "wrongAdmin"),
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

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}
		})
	}
}
