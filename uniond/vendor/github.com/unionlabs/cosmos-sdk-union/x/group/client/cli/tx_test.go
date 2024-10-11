package cli_test

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"testing"

	"github.com/stretchr/testify/suite"

	// without this import amino json encoding will fail when resolving any types
	_ "cosmossdk.io/api/cosmos/group/v1"
	sdkmath "cosmossdk.io/math"
	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/group"
	groupcli "cosmossdk.io/x/group/client/cli"
	groupmodule "cosmossdk.io/x/group/module"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	sdk "github.com/cosmos/cosmos-sdk/types"
	testutilmod "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

var validMetadata = "metadata"

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
	s.encCfg = testutilmod.MakeTestEncodingConfig(codectestutil.CodecOptions{}, groupmodule.AppModule{})
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

	s.commonFlags = []string{
		fmt.Sprintf("--%s=true", flags.FlagSkipConfirmation),
		fmt.Sprintf("--%s=%s", flags.FlagBroadcastMode, flags.BroadcastSync),
		fmt.Sprintf("--%s=%s", flags.FlagFees, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))).String()),
	}

	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)
	val := accounts[0]
	valAddr, err := s.baseCtx.AddressCodec.BytesToString(val.Address)
	s.Require().NoError(err)

	ctxGen := func() client.Context {
		bz, _ := s.encCfg.Codec.Marshal(&sdk.TxResponse{})
		c := clitestutil.NewMockCometRPCWithResponseQueryValue(bz)
		return s.baseCtx.WithClient(c)
	}
	s.clientCtx = ctxGen()

	// create a new account
	info, _, err := s.clientCtx.Keyring.NewMnemonic("NewValidator", keyring.English, sdk.FullFundraiserPath, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	s.Require().NoError(err)

	pk, err := info.GetPubKey()
	s.Require().NoError(err)

	account := sdk.AccAddress(pk.Address())

	from := val.Address
	coins := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(2000)))

	fromAddr, err := s.baseCtx.AddressCodec.BytesToString(from)
	s.Require().NoError(err)
	toAddr, err := s.baseCtx.AddressCodec.BytesToString(account)
	s.Require().NoError(err)

	msgSend := &banktypes.MsgSend{
		FromAddress: fromAddr,
		ToAddress:   toAddr,
		Amount:      coins,
	}

	_, err = clitestutil.SubmitTestTx(s.clientCtx, msgSend, from, clitestutil.TestTxConfig{})
	s.Require().NoError(err)

	// create a group
	validMembers := fmt.Sprintf(`
	{
		"members": [
			{
				"address": "%s",
				"weight": "3",
				"metadata": "%s"
			}
		]
	}`, valAddr, validMetadata)
	validMembersFile := testutil.WriteToNewTempFile(s.T(), validMembers)
	out, err := clitestutil.ExecTestCLICmd(s.clientCtx, groupcli.MsgCreateGroupCmd(),
		append(
			[]string{
				valAddr,
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

	s.group = &group.GroupInfo{Id: 1, Admin: valAddr, Metadata: validMetadata, TotalWeight: "3", Version: 1}
}

func (s *CLITestSuite) TestTxCreateGroup() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)
	account0Addr, err := s.baseCtx.AddressCodec.BytesToString(accounts[0].Address)
	s.Require().NoError(err)

	cmd := groupcli.MsgCreateGroupCmd()
	cmd.SetOutput(io.Discard)

	validMembers := fmt.Sprintf(`{"members": [{
		"address": "%s",
		  "weight": "1",
		  "metadata": "%s"
	  }]}`, account0Addr, validMetadata)
	validMembersFile := testutil.WriteToNewTempFile(s.T(), validMembers)

	invalidMembersWeight := fmt.Sprintf(`{"members": [{
			"address": "%s",
			  "weight": "0"
		  }]}`, account0Addr)
	invalidMembersWeightFile := testutil.WriteToNewTempFile(s.T(), invalidMembersWeight)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
		expectErrMsg string
	}{
		{
			name: "correct data",
			args: append(
				[]string{
					account0Addr,
					"",
					validMembersFile.Name(),
				},
				s.commonFlags...,
			),
			expCmdOutput: fmt.Sprintf("%s %s %s", account0Addr, "", validMembersFile.Name()),
			expectErrMsg: "",
		},
		{
			"with amino-json",
			append(
				[]string{
					account0Addr,
					"",
					validMembersFile.Name(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", account0Addr, "", validMembersFile.Name()),
			"",
		},
		{
			"invalid members weight",
			append(
				[]string{
					account0Addr,
					"null",
					invalidMembersWeightFile.Name(),
				},
				s.commonFlags...,
			),
			"",
			"weight must be positive",
		},
		{
			"no member provided",
			append(
				[]string{
					account0Addr,
					"null",
					"doesnotexist.json",
				},
				s.commonFlags...,
			),
			"",
			"no such file or directory",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			ctx := svrcmd.CreateExecuteContext(context.Background())
			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)
			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(s.baseCtx, cmd, tc.args)
			if tc.expectErrMsg != "" {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err)
				msg := &sdk.TxResponse{}
				s.Require().NoError(s.baseCtx.Codec.UnmarshalJSON(out.Bytes(), msg), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxUpdateGroupMembers() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 3)
	groupPolicyAddress := accounts[2]

	account0Addr, err := s.baseCtx.AddressCodec.BytesToString(accounts[0].Address)
	s.Require().NoError(err)

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
		  "weight": "-1",
		  "metadata": "foo"
	  }]}`, accounts[1])
	invalidMembersMetadataFileName := testutil.WriteToNewTempFile(s.T(), invalidMembersMetadata).Name()

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
		expectErrMsg string
	}{
		{
			"correct data",
			append(
				[]string{
					account0Addr,
					groupID,
					validUpdatedMembersFileName,
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", account0Addr, groupID, validUpdatedMembersFileName),
			"",
		},
		{
			"with amino-json",
			append(
				[]string{
					account0Addr,
					groupID,
					validUpdatedMembersFileName,
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s --%s=%s", account0Addr, groupID, validUpdatedMembersFileName, flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
			"",
		},
		{
			"group id invalid",
			append(
				[]string{
					account0Addr,
					"0",
					validUpdatedMembersFileName,
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", account0Addr, "0", validUpdatedMembersFileName),
			"group id cannot be 0",
		},
		{
			"group member weight invalid",
			append(
				[]string{
					account0Addr,
					groupID,
					invalidMembersMetadataFileName,
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s %s %s", account0Addr, groupID, invalidMembersMetadataFileName),
			"invalid weight -1",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(s.baseCtx, cmd, tc.args)
			if tc.expectErrMsg != "" {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err)
				msg := &sdk.TxResponse{}
				s.Require().NoError(s.baseCtx.Codec.UnmarshalJSON(out.Bytes(), msg), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxCreateGroupWithPolicy() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 1)

	account0Addr, err := s.baseCtx.AddressCodec.BytesToString(accounts[0].Address)
	s.Require().NoError(err)

	cmd := groupcli.MsgCreateGroupWithPolicyCmd()
	cmd.SetOutput(io.Discard)

	validMembers := fmt.Sprintf(`{"members": [{
		"address": "%s",
		  "weight": "1",
		  "metadata": "%s"
	}]}`, account0Addr, validMetadata)
	validMembersFile := testutil.WriteToNewTempFile(s.T(), validMembers)

	invalidMembersWeight := fmt.Sprintf(`{"members": [{
		"address": "%s",
		  "weight": "0"
	}]}`, account0Addr)
	invalidMembersWeightFile := testutil.WriteToNewTempFile(s.T(), invalidMembersWeight)

	thresholdDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type": "/cosmos.group.v1.ThresholdDecisionPolicy","threshold": "1","windows": {"voting_period":"1s"}}`)

	testCases := []struct {
		name         string
		args         []string
		expectErrMsg string
		expCmdOutput string
	}{
		{
			"correct data",
			append(
				[]string{
					account0Addr,
					validMetadata,
					validMetadata,
					validMembersFile.Name(),
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%v", groupcli.FlagGroupPolicyAsAdmin, false),
				},
				s.commonFlags...,
			),
			"",
			fmt.Sprintf("%s %s %s %s %s --%s=%v", account0Addr, validMetadata, validMetadata, validMembersFile.Name(), thresholdDecisionPolicyFile.Name(), groupcli.FlagGroupPolicyAsAdmin, false),
		},
		{
			"group-policy-as-admin is true",
			append(
				[]string{
					account0Addr,
					validMetadata,
					validMetadata,
					validMembersFile.Name(),
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%v", groupcli.FlagGroupPolicyAsAdmin, true),
				},
				s.commonFlags...,
			),
			"",
			fmt.Sprintf("%s %s %s %s %s --%s=%v", account0Addr, validMetadata, validMetadata, validMembersFile.Name(), thresholdDecisionPolicyFile.Name(), groupcli.FlagGroupPolicyAsAdmin, true),
		},
		{
			"with amino-json",
			append(
				[]string{
					account0Addr,
					validMetadata,
					validMetadata,
					validMembersFile.Name(),
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%v", groupcli.FlagGroupPolicyAsAdmin, false),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			"",
			fmt.Sprintf("%s %s %s %s %s --%s=%v --%s=%s", account0Addr, validMetadata, validMetadata, validMembersFile.Name(), thresholdDecisionPolicyFile.Name(), groupcli.FlagGroupPolicyAsAdmin, false, flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"invalid members weight",
			append(
				[]string{
					account0Addr,
					validMetadata,
					validMetadata,
					invalidMembersWeightFile.Name(),
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%v", groupcli.FlagGroupPolicyAsAdmin, false),
				},
				s.commonFlags...,
			),
			"weight must be positive",
			fmt.Sprintf("%s %s %s %s %s --%s=%v", account0Addr, validMetadata, validMetadata, invalidMembersWeightFile.Name(), thresholdDecisionPolicyFile.Name(), groupcli.FlagGroupPolicyAsAdmin, false),
		},
	}
	for _, tc := range testCases {
		s.Run(tc.name, func() {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(s.baseCtx, cmd, tc.args)
			if tc.expectErrMsg != "" {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err, out.String())
				msg := &sdk.TxResponse{}
				s.Require().NoError(s.baseCtx.Codec.UnmarshalJSON(out.Bytes(), msg), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxCreateGroupPolicy() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 2)
	valAddr, err := s.baseCtx.AddressCodec.BytesToString(accounts[0].Address)
	s.Require().NoError(err)

	groupID := s.group.Id

	thresholdDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type": "/cosmos.group.v1.ThresholdDecisionPolicy","threshold": "1","windows": {"voting_period":"1s"}}`)

	percentageDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"0.5", "windows":{"voting_period":"1s"}}`)
	invalidNegativePercentageDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"-0.5", "windows":{"voting_period":"1s"}}`)
	invalidPercentageDecisionPolicyFile := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"2", "windows":{"voting_period":"1s"}}`)

	cmd := groupcli.MsgCreateGroupPolicyCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		args         []string
		expectErrMsg string
		expCmdOutput string
	}{
		{
			"correct data",
			append(
				[]string{
					valAddr,
					fmt.Sprintf("%v", groupID),
					validMetadata,
					thresholdDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			"",
			fmt.Sprintf("%s %s %s %s", valAddr, fmt.Sprintf("%v", groupID), validMetadata, thresholdDecisionPolicyFile.Name()),
		},
		{
			"correct data with percentage decision policy",
			append(
				[]string{
					valAddr,
					fmt.Sprintf("%v", groupID),
					validMetadata,
					percentageDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			"",
			fmt.Sprintf("%s %s %s %s", valAddr, fmt.Sprintf("%v", groupID), validMetadata, percentageDecisionPolicyFile.Name()),
		},
		{
			"with amino-json",
			append(
				[]string{
					valAddr,
					fmt.Sprintf("%v", groupID),
					validMetadata,
					thresholdDecisionPolicyFile.Name(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			"",
			fmt.Sprintf("%s %s %s %s --%s=%s", valAddr, fmt.Sprintf("%v", groupID), validMetadata, thresholdDecisionPolicyFile.Name(), flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
		},
		{
			"wrong admin",
			append(
				[]string{
					"wrongAdmin",
					fmt.Sprintf("%v", groupID),
					validMetadata,
					thresholdDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			"key not found",
			fmt.Sprintf("%s %s %s %s", "wrongAdmin", fmt.Sprintf("%v", groupID), validMetadata, thresholdDecisionPolicyFile.Name()),
		},
		{
			"invalid percentage decision policy with negative value",
			append(
				[]string{
					valAddr,
					fmt.Sprintf("%v", groupID),
					validMetadata,
					invalidNegativePercentageDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			"expected a positive decimal",
			fmt.Sprintf("%s %s %s %s", valAddr, fmt.Sprintf("%v", groupID), validMetadata, invalidNegativePercentageDecisionPolicyFile.Name()),
		},
		{
			"invalid percentage decision policy with value greater than 1",
			append(
				[]string{
					valAddr,
					fmt.Sprintf("%v", groupID),
					validMetadata,
					invalidPercentageDecisionPolicyFile.Name(),
				},
				s.commonFlags...,
			),
			"percentage must be > 0 and <= 1",
			fmt.Sprintf("%s %s %s %s", valAddr, fmt.Sprintf("%v", groupID), validMetadata, invalidPercentageDecisionPolicyFile.Name()),
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(s.baseCtx, cmd, tc.args)
			if tc.expectErrMsg != "" {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err, out.String())
				msg := &sdk.TxResponse{}
				s.Require().NoError(s.baseCtx.Codec.UnmarshalJSON(out.Bytes(), msg), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxUpdateGroupPolicyDecisionPolicy() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 3)
	newAdmin, err := s.baseCtx.AddressCodec.BytesToString(accounts[0].Address)
	s.Require().NoError(err)
	groupPolicyAdmin, err := s.baseCtx.AddressCodec.BytesToString(accounts[1].Address)
	s.Require().NoError(err)
	groupPolicyAddress, err := s.baseCtx.AddressCodec.BytesToString(accounts[2].Address)
	s.Require().NoError(err)

	commonFlags := s.commonFlags
	commonFlags = append(commonFlags, fmt.Sprintf("--%s=%d", flags.FlagGas, 300000))

	thresholdDecisionPolicy := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.ThresholdDecisionPolicy", "threshold":"1", "windows":{"voting_period":"40000s"}}`)
	percentageDecisionPolicy := testutil.WriteToNewTempFile(s.T(), `{"@type":"/cosmos.group.v1.PercentageDecisionPolicy", "percentage":"0.5", "windows":{"voting_period":"40000s"}}`)

	cmd := groupcli.MsgUpdateGroupPolicyDecisionPolicyCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
		expectErrMsg string
	}{
		{
			"correct data",
			append(
				[]string{
					groupPolicyAdmin,
					groupPolicyAddress,
					thresholdDecisionPolicy.Name(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin, groupPolicyAddress, thresholdDecisionPolicy.Name()),
			"",
		},
		{
			"correct data with percentage decision policy",
			append(
				[]string{
					groupPolicyAdmin,
					groupPolicyAddress,
					percentageDecisionPolicy.Name(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", groupPolicyAdmin, groupPolicyAddress, percentageDecisionPolicy.Name()),
			"",
		},
		{
			"with amino-json",
			append(
				[]string{
					groupPolicyAdmin,
					groupPolicyAddress,
					thresholdDecisionPolicy.Name(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s --%s=%s", groupPolicyAdmin, groupPolicyAddress, thresholdDecisionPolicy.Name(), flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
			"",
		},
		{
			"wrong admin",
			append(
				[]string{
					newAdmin,
					"invalid",
					thresholdDecisionPolicy.Name(),
				},
				commonFlags...,
			),
			fmt.Sprintf("%s %s %s", newAdmin, "invalid", thresholdDecisionPolicy.Name()),
			"decoding bech32 failed",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(s.baseCtx, cmd, tc.args)
			if tc.expectErrMsg != "" {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err)
				msg := &sdk.TxResponse{}
				s.Require().NoError(s.baseCtx.Codec.UnmarshalJSON(out.Bytes(), msg), out.String())
			}
		})
	}
}

func (s *CLITestSuite) TestTxSubmitProposal() {
	accounts := testutil.CreateKeyringAccounts(s.T(), s.kr, 2)

	groupPolicyAddress, err := s.baseCtx.AddressCodec.BytesToString(accounts[1].Address)
	s.Require().NoError(err)
	account0Addr, err := s.baseCtx.AddressCodec.BytesToString(accounts[0].Address)
	s.Require().NoError(err)

	p := groupcli.Proposal{
		GroupPolicyAddress: groupPolicyAddress,
		Messages:           []json.RawMessage{},
		Metadata:           validMetadata,
		Proposers:          []string{account0Addr},
	}
	bz, err := json.Marshal(&p)
	s.Require().NoError(err)
	proposalFile := testutil.WriteToNewTempFile(s.T(), string(bz))

	cmd := groupcli.MsgSubmitProposalCmd()
	cmd.SetOutput(io.Discard)

	testCases := []struct {
		name         string
		args         []string
		expCmdOutput string
		expectErrMsg string
	}{
		{
			"correct data",
			append(
				[]string{
					proposalFile.Name(),
				},
				s.commonFlags...,
			),
			proposalFile.Name(),
			"",
		},
		{
			"with try exec",
			append(
				[]string{
					proposalFile.Name(),
					fmt.Sprintf("--%s=try", groupcli.FlagExec),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s --%s=try", proposalFile.Name(), groupcli.FlagExec),
			"",
		},
		{
			"with try exec, not enough yes votes for proposal to pass",
			append(
				[]string{
					proposalFile.Name(),
					fmt.Sprintf("--%s=try", groupcli.FlagExec),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s --%s=try", proposalFile.Name(), groupcli.FlagExec),
			"",
		},
		{
			"with amino-json",
			append(
				[]string{
					proposalFile.Name(),
					fmt.Sprintf("--%s=%s", flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
				},
				s.commonFlags...,
			),
			fmt.Sprintf("%s --%s=%s", proposalFile.Name(), flags.FlagSignMode, flags.SignModeLegacyAminoJSON),
			"",
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			ctx := svrcmd.CreateExecuteContext(context.Background())

			cmd.SetContext(ctx)
			cmd.SetArgs(tc.args)

			s.Require().NoError(client.SetCmdClientContextHandler(s.baseCtx, cmd))

			if len(tc.args) != 0 {
				s.Require().Contains(fmt.Sprint(cmd), tc.expCmdOutput)
			}

			out, err := clitestutil.ExecTestCLICmd(s.baseCtx, cmd, tc.args)
			if tc.expectErrMsg != "" {
				s.Require().Error(err)
				s.Require().Contains(out.String(), tc.expectErrMsg)
			} else {
				s.Require().NoError(err)
				msg := &sdk.TxResponse{}
				s.Require().NoError(s.baseCtx.Codec.UnmarshalJSON(out.Bytes(), msg), out.String())
			}
		})
	}
}
