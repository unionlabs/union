package keeper_test

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	coreaddress "cosmossdk.io/core/address"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/keeper"
	"cosmossdk.io/x/group/module"
	grouptestutil "cosmossdk.io/x/group/testutil"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

type GenesisTestSuite struct {
	suite.Suite

	ctx          context.Context
	sdkCtx       sdk.Context
	keeper       keeper.Keeper
	cdc          *codec.ProtoCodec
	addressCodec coreaddress.Codec
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
	key := storetypes.NewKVStoreKey(group.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	testCtx := testutil.DefaultContextWithDB(s.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{})

	ctrl := gomock.NewController(s.T())
	accountKeeper := grouptestutil.NewMockAccountKeeper(ctrl)
	accountKeeper.EXPECT().GetAccount(gomock.Any(), accAddr).Return(authtypes.NewBaseAccountWithAddress(accAddr)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), memberAddr).Return(authtypes.NewBaseAccountWithAddress(memberAddr)).AnyTimes()
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	bApp := baseapp.NewBaseApp(
		"group",
		log.NewNopLogger(),
		testCtx.DB,
		encCfg.TxConfig.TxDecoder(),
	)

	banktypes.RegisterInterfaces(encCfg.InterfaceRegistry)

	s.sdkCtx = testCtx.Ctx
	s.cdc = codec.NewProtoCodec(encCfg.InterfaceRegistry)
	s.ctx = s.sdkCtx
	s.addressCodec = address.NewBech32Codec("cosmos")

	env := runtime.NewEnvironment(storeService, log.NewNopLogger(), runtime.EnvWithQueryRouterService(bApp.GRPCQueryRouter()), runtime.EnvWithMsgRouterService(bApp.MsgServiceRouter()))
	s.keeper = keeper.NewKeeper(env, s.cdc, accountKeeper, group.DefaultConfig())
}

func (s *GenesisTestSuite) TestInitExportGenesis() {
	sdkCtx := s.sdkCtx
	ctx := s.ctx
	cdc := s.cdc

	submittedAt := time.Now().UTC()
	timeout := submittedAt.Add(time.Second * 1).UTC()

	accStrAddr, err := s.addressCodec.BytesToString(accAddr)
	s.Require().NoError(err)
	memberStrAddr, err := s.addressCodec.BytesToString(memberAddr)
	s.Require().NoError(err)

	groupPolicy := &group.GroupPolicyInfo{
		Address:  accStrAddr,
		GroupId:  1,
		Admin:    accStrAddr,
		Version:  1,
		Metadata: "policy metadata",
	}
	err = groupPolicy.SetDecisionPolicy(&group.ThresholdDecisionPolicy{
		Threshold: "1",
		Windows: &group.DecisionPolicyWindows{
			VotingPeriod: time.Second,
		},
	})
	s.Require().NoError(err)

	proposal := &group.Proposal{
		Id:                 1,
		GroupPolicyAddress: accStrAddr,
		Metadata:           "proposal metadata",
		GroupVersion:       1,
		GroupPolicyVersion: 1,
		Proposers: []string{
			memberStrAddr,
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
		FromAddress: accStrAddr,
		ToAddress:   memberStrAddr,
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}})
	s.Require().NoError(err)

	genesisState := &group.GenesisState{
		GroupSeq:       2,
		Groups:         []*group.GroupInfo{{Id: 1, Admin: accStrAddr, Metadata: "1", Version: 1, TotalWeight: "1"}, {Id: 2, Admin: accStrAddr, Metadata: "2", Version: 2, TotalWeight: "2"}},
		GroupMembers:   []*group.GroupMember{{GroupId: 1, Member: &group.Member{Address: memberStrAddr, Weight: "1", Metadata: "member metadata"}}, {GroupId: 2, Member: &group.Member{Address: memberStrAddr, Weight: "2", Metadata: "member metadata"}}},
		GroupPolicySeq: 1,
		GroupPolicies:  []*group.GroupPolicyInfo{groupPolicy},
		ProposalSeq:    1,
		Proposals:      []*group.Proposal{proposal},
		Votes:          []*group.Vote{{ProposalId: proposal.Id, Voter: memberStrAddr, SubmitTime: submittedAt, Option: group.VOTE_OPTION_YES}},
	}
	genesisBytes, err := cdc.MarshalJSON(genesisState)
	s.Require().NoError(err)

	genesisData := map[string]json.RawMessage{
		group.ModuleName: genesisBytes,
	}

	err = s.keeper.InitGenesis(sdkCtx, cdc, genesisData[group.ModuleName])
	s.Require().NoError(err)

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

	exported, err := s.keeper.ExportGenesis(sdkCtx, cdc)
	s.Require().NoError(err)
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

func (s *GenesisTestSuite) assertGroupPoliciesEqual(g, other *group.GroupPolicyInfo) {
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

func (s *GenesisTestSuite) assertProposalsEqual(g, other *group.Proposal) {
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
