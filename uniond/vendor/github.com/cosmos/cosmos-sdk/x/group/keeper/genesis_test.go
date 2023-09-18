package keeper_test

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"

	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/group"
	"github.com/cosmos/cosmos-sdk/x/group/keeper"
	"github.com/cosmos/cosmos-sdk/x/group/module"
	grouptestutil "github.com/cosmos/cosmos-sdk/x/group/testutil"
)

type GenesisTestSuite struct {
	suite.Suite

	ctx    context.Context
	sdkCtx sdk.Context
	keeper keeper.Keeper
	cdc    *codec.ProtoCodec
}

func TestGenesisTestSuite(t *testing.T) {
	suite.Run(t, new(GenesisTestSuite))
}

var (
	memberPub  = secp256k1.GenPrivKey().PubKey()
	accPub     = secp256k1.GenPrivKey().PubKey()
	accAddr    = sdk.AccAddress(accPub.Address())
	memberAddr = sdk.AccAddress(memberPub.Address())
)

func (s *GenesisTestSuite) SetupTest() {
	key := sdk.NewKVStoreKey(group.StoreKey)
	testCtx := testutil.DefaultContextWithDB(s.T(), key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(module.AppModuleBasic{})

	ctrl := gomock.NewController(s.T())
	accountKeeper := grouptestutil.NewMockAccountKeeper(ctrl)
	accountKeeper.EXPECT().GetAccount(gomock.Any(), accAddr).Return(authtypes.NewBaseAccountWithAddress(accAddr)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), memberAddr).Return(authtypes.NewBaseAccountWithAddress(memberAddr)).AnyTimes()

	bApp := baseapp.NewBaseApp(
		"group",
		log.NewNopLogger(),
		testCtx.DB,
		encCfg.TxConfig.TxDecoder(),
	)

	banktypes.RegisterInterfaces(encCfg.InterfaceRegistry)

	s.sdkCtx = testCtx.Ctx
	s.cdc = codec.NewProtoCodec(encCfg.InterfaceRegistry)
	s.ctx = sdk.WrapSDKContext(s.sdkCtx)

	s.keeper = keeper.NewKeeper(key, s.cdc, bApp.MsgServiceRouter(), accountKeeper, group.DefaultConfig())
}

func (s *GenesisTestSuite) TestInitExportGenesis() {
	sdkCtx := s.sdkCtx
	ctx := s.ctx
	cdc := s.cdc

	submittedAt := time.Now().UTC()
	timeout := submittedAt.Add(time.Second * 1).UTC()

	groupPolicy := &group.GroupPolicyInfo{
		Address:  accAddr.String(),
		GroupId:  1,
		Admin:    accAddr.String(),
		Version:  1,
		Metadata: "policy metadata",
	}
	err := groupPolicy.SetDecisionPolicy(&group.ThresholdDecisionPolicy{
		Threshold: "1",
		Windows: &group.DecisionPolicyWindows{
			VotingPeriod: time.Second,
		},
	})
	s.Require().NoError(err)

	proposal := &group.Proposal{
		Id:                 1,
		GroupPolicyAddress: accAddr.String(),
		Metadata:           "proposal metadata",
		GroupVersion:       1,
		GroupPolicyVersion: 1,
		Proposers: []string{
			memberAddr.String(),
		},
		SubmitTime: submittedAt,
		Status:     group.PROPOSAL_STATUS_ACCEPTED,
		FinalTallyResult: group.TallyResult{
			YesCount:        "1",
			NoCount:         "0",
			AbstainCount:    "0",
			NoWithVetoCount: "0",
		},
		VotingPeriodEnd: timeout,
		ExecutorResult:  group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
	}
	err = proposal.SetMsgs([]sdk.Msg{&banktypes.MsgSend{
		FromAddress: accAddr.String(),
		ToAddress:   memberAddr.String(),
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}})
	s.Require().NoError(err)

	genesisState := &group.GenesisState{
		GroupSeq:       2,
		Groups:         []*group.GroupInfo{{Id: 1, Admin: accAddr.String(), Metadata: "1", Version: 1, TotalWeight: "1"}, {Id: 2, Admin: accAddr.String(), Metadata: "2", Version: 2, TotalWeight: "2"}},
		GroupMembers:   []*group.GroupMember{{GroupId: 1, Member: &group.Member{Address: memberAddr.String(), Weight: "1", Metadata: "member metadata"}}, {GroupId: 2, Member: &group.Member{Address: memberAddr.String(), Weight: "2", Metadata: "member metadata"}}},
		GroupPolicySeq: 1,
		GroupPolicies:  []*group.GroupPolicyInfo{groupPolicy},
		ProposalSeq:    1,
		Proposals:      []*group.Proposal{proposal},
		Votes:          []*group.Vote{{ProposalId: proposal.Id, Voter: memberAddr.String(), SubmitTime: submittedAt, Option: group.VOTE_OPTION_YES}},
	}
	genesisBytes, err := cdc.MarshalJSON(genesisState)
	s.Require().NoError(err)

	genesisData := map[string]json.RawMessage{
		group.ModuleName: genesisBytes,
	}

	s.keeper.InitGenesis(sdkCtx, cdc, genesisData[group.ModuleName])

	for i, g := range genesisState.Groups {
		res, err := s.keeper.GroupInfo(ctx, &group.QueryGroupInfoRequest{
			GroupId: g.Id,
		})
		s.Require().NoError(err)
		s.Require().Equal(g, res.Info)

		membersRes, err := s.keeper.GroupMembers(ctx, &group.QueryGroupMembersRequest{
			GroupId: g.Id,
		})
		s.Require().NoError(err)
		s.Require().Equal(len(membersRes.Members), 1)
		s.Require().Equal(membersRes.Members[0], genesisState.GroupMembers[i])
	}

	for _, g := range genesisState.GroupPolicies {
		res, err := s.keeper.GroupPolicyInfo(ctx, &group.QueryGroupPolicyInfoRequest{
			Address: g.Address,
		})
		s.Require().NoError(err)
		s.assertGroupPoliciesEqual(g, res.Info)
	}

	for _, g := range genesisState.Proposals {
		res, err := s.keeper.Proposal(ctx, &group.QueryProposalRequest{
			ProposalId: g.Id,
		})
		s.Require().NoError(err)
		s.assertProposalsEqual(g, res.Proposal)

		votesRes, err := s.keeper.VotesByProposal(ctx, &group.QueryVotesByProposalRequest{
			ProposalId: g.Id,
		})
		s.Require().NoError(err)
		s.Require().Equal(len(votesRes.Votes), 1)
		s.Require().Equal(votesRes.Votes[0], genesisState.Votes[0])
	}

	exported := s.keeper.ExportGenesis(sdkCtx, cdc)
	bz, err := cdc.MarshalJSON(exported)
	s.Require().NoError(err)

	var exportedGenesisState group.GenesisState
	err = cdc.UnmarshalJSON(bz, &exportedGenesisState)
	s.Require().NoError(err)

	s.Require().Equal(genesisState.Groups, exportedGenesisState.Groups)
	s.Require().Equal(genesisState.GroupMembers, exportedGenesisState.GroupMembers)

	s.Require().Equal(len(genesisState.GroupPolicies), len(exportedGenesisState.GroupPolicies))
	for i, g := range genesisState.GroupPolicies {
		res := exportedGenesisState.GroupPolicies[i]
		s.Require().NoError(err)
		s.assertGroupPoliciesEqual(g, res)
	}

	s.Require().Equal(len(genesisState.Proposals), len(exportedGenesisState.Proposals))
	for i, g := range genesisState.Proposals {
		res := exportedGenesisState.Proposals[i]
		s.Require().NoError(err)
		s.assertProposalsEqual(g, res)
	}
	s.Require().Equal(genesisState.Votes, exportedGenesisState.Votes)

	s.Require().Equal(genesisState.GroupSeq, exportedGenesisState.GroupSeq)
	s.Require().Equal(genesisState.GroupPolicySeq, exportedGenesisState.GroupPolicySeq)
	s.Require().Equal(genesisState.ProposalSeq, exportedGenesisState.ProposalSeq)
}

func (s *GenesisTestSuite) assertGroupPoliciesEqual(g *group.GroupPolicyInfo, other *group.GroupPolicyInfo) {
	require := s.Require()
	require.Equal(g.Address, other.Address)
	require.Equal(g.GroupId, other.GroupId)
	require.Equal(g.Admin, other.Admin)
	require.Equal(g.Metadata, other.Metadata)
	require.Equal(g.Version, other.Version)
	dp1, err := g.GetDecisionPolicy()
	require.NoError(err)
	dp2, err := other.GetDecisionPolicy()
	require.NoError(err)
	require.Equal(dp1, dp2)
}

func (s *GenesisTestSuite) assertProposalsEqual(g *group.Proposal, other *group.Proposal) {
	require := s.Require()
	require.Equal(g.Id, other.Id)
	require.Equal(g.GroupPolicyAddress, other.GroupPolicyAddress)
	require.Equal(g.Metadata, other.Metadata)
	require.Equal(g.Proposers, other.Proposers)
	require.Equal(g.SubmitTime, other.SubmitTime)
	require.Equal(g.GroupVersion, other.GroupVersion)
	require.Equal(g.GroupPolicyVersion, other.GroupPolicyVersion)
	require.Equal(g.Status, other.Status)
	require.Equal(g.FinalTallyResult, other.FinalTallyResult)
	require.Equal(g.VotingPeriodEnd, other.VotingPeriodEnd)
	require.Equal(g.ExecutorResult, other.ExecutorResult)
	msgs1, err := g.GetMsgs()
	require.NoError(err)
	msgs2, err := other.GetMsgs()
	require.NoError(err)
	require.Equal(msgs1, msgs2)
}
