package types_test

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

type caseAny struct {
	name    string
	any     *codectypes.Any
	expPass bool
}

func (suite *TypesTestSuite) TestPackClientState() {
	testCases := []struct {
		name        string
		clientState exported.ClientState
		expPass     bool
	}{
		{
			"solo machine client",
			ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "solomachine", "", 2).ClientState(),
			true,
		},
		{
			"tendermint client",
			ibctm.NewClientState(suite.chainA.ChainID, ibctesting.DefaultTrustLevel, ibctesting.TrustingPeriod, ibctesting.UnbondingPeriod, ibctesting.MaxClockDrift, clientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath),
			true,
		},
		{
			"nil",
			nil,
			false,
		},
	}

	testCasesAny := []caseAny{}

	for _, tc := range testCases {
		tc := tc
		protoAny, err := types.PackClientState(tc.clientState)
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}

		testCasesAny = append(testCasesAny, caseAny{tc.name, protoAny, tc.expPass})
	}

	for i, tc := range testCasesAny {
		i, tc := i, tc

		cs, err := types.UnpackClientState(tc.any)
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
			suite.Require().Equal(testCases[i].clientState, cs, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}
	}
}

func (suite *TypesTestSuite) TestPackConsensusState() {
	testCases := []struct {
		name           string
		consensusState exported.ConsensusState
		expPass        bool
	}{
		{
			"solo machine consensus",
			ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "solomachine", "", 2).ConsensusState(),
			true,
		},
		{
			"tendermint consensus",
			suite.chainA.LastHeader.ConsensusState(),
			true,
		},
		{
			"nil",
			nil,
			false,
		},
	}

	testCasesAny := []caseAny{}

	for _, tc := range testCases {
		tc := tc
		protoAny, err := types.PackConsensusState(tc.consensusState)
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}
		testCasesAny = append(testCasesAny, caseAny{tc.name, protoAny, tc.expPass})
	}

	for i, tc := range testCasesAny {
		tc := tc

		cs, err := types.UnpackConsensusState(tc.any)
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
			suite.Require().Equal(testCases[i].consensusState, cs, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}
	}
}

func (suite *TypesTestSuite) TestPackClientMessage() {
	testCases := []struct {
		name          string
		clientMessage exported.ClientMessage
		expPass       bool
	}{
		{
			"solo machine header",
			ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "solomachine", "", 2).CreateHeader("solomachine"),
			true,
		},
		{
			"tendermint header",
			suite.chainA.LastHeader,
			true,
		},
		{
			"nil",
			nil,
			false,
		},
	}

	testCasesAny := []caseAny{}

	for _, tc := range testCases {
		tc := tc
		protoAny, err := types.PackClientMessage(tc.clientMessage)
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}

		testCasesAny = append(testCasesAny, caseAny{tc.name, protoAny, tc.expPass})
	}

	for i, tc := range testCasesAny {
		tc := tc
		cs, err := types.UnpackClientMessage(tc.any)
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
			suite.Require().Equal(testCases[i].clientMessage, cs, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}
	}
}

func (suite *TypesTestSuite) TestCodecTypeRegistration() {
	testCases := []struct {
		name    string
		typeURL string
		expPass bool
	}{
		{
			"success: MsgCreateClient",
			sdk.MsgTypeURL(&types.MsgCreateClient{}),
			true,
		},
		{
			"success: MsgUpdateClient",
			sdk.MsgTypeURL(&types.MsgUpdateClient{}),
			true,
		},
		{
			"success: MsgUpgradeClient",
			sdk.MsgTypeURL(&types.MsgUpgradeClient{}),
			true,
		},
		{
			"success: MsgSubmitMisbehaviour",
			sdk.MsgTypeURL(&types.MsgSubmitMisbehaviour{}),
			true,
		},
		{
			"success: MsgRecoverClient",
			sdk.MsgTypeURL(&types.MsgRecoverClient{}),
			true,
		},
		{
			"success: MsgIBCSoftwareUpgrade",
			sdk.MsgTypeURL(&types.MsgIBCSoftwareUpgrade{}),
			true,
		},
		{
			"success: MsgUpdateParams",
			sdk.MsgTypeURL(&types.MsgUpdateParams{}),
			true,
		},
		{
			"success: ClientUpdateProposal",
			sdk.MsgTypeURL(&types.ClientUpdateProposal{}),
			true,
		},
		{
			"success: UpgradeProposal",
			sdk.MsgTypeURL(&types.UpgradeProposal{}),
			true,
		},
		{
			"type not registered on codec",
			"ibc.invalid.MsgTypeURL",
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			msg, err := suite.chainA.GetSimApp().AppCodec().InterfaceRegistry().Resolve(tc.typeURL)

			if tc.expPass {
				suite.Require().NotNil(msg)
				suite.Require().NoError(err)
			} else {
				suite.Require().Nil(msg)
				suite.Require().Error(err)
			}
		})
	}
}
