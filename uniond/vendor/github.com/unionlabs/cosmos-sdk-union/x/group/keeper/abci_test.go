package keeper_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/address"
	"cosmossdk.io/core/header"
	"cosmossdk.io/depinject"
	"cosmossdk.io/log"
	"cosmossdk.io/math"
	bankkeeper "cosmossdk.io/x/bank/keeper"
	"cosmossdk.io/x/bank/testutil"
	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/keeper"
	grouptestutil "cosmossdk.io/x/group/testutil"
	stakingkeeper "cosmossdk.io/x/staking/keeper"

	codecaddress "github.com/cosmos/cosmos-sdk/codec/address"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type IntegrationTestSuite struct {
	suite.Suite

	app               *runtime.App
	ctx               sdk.Context
	addrs             []sdk.AccAddress
	groupKeeper       keeper.Keeper
	bankKeeper        bankkeeper.Keeper
	stakingKeeper     *stakingkeeper.Keeper
	interfaceRegistry codectypes.InterfaceRegistry

	addressCodec address.Codec
}

func TestIntegrationTestSuite(t *testing.T) {
	suite.Run(t, new(IntegrationTestSuite))
}

func (s *IntegrationTestSuite) SetupTest() {
	app, err := simtestutil.Setup(
		depinject.Configs(
			grouptestutil.AppConfig,
			depinject.Supply(log.NewNopLogger()),
		),
		&s.interfaceRegistry,
		&s.bankKeeper,
		&s.stakingKeeper,
		&s.groupKeeper,
	)
	s.Require().NoError(err)

	ctx := app.BaseApp.NewContext(false)

	ctx = ctx.WithHeaderInfo(header.Info{Time: time.Now()})

	s.ctx = ctx

	s.addrs = simtestutil.AddTestAddrsIncremental(s.bankKeeper, s.stakingKeeper, ctx, 4, math.NewInt(30000000))

	s.addressCodec = codecaddress.NewBech32Codec("cosmos")
}

func (s *IntegrationTestSuite) TestEndBlockerPruning() {
	ctx := s.ctx
	addr1, err := s.addressCodec.BytesToString(s.addrs[0])
	s.Require().NoError(err)
	addr2, err := s.addressCodec.BytesToString(s.addrs[1])
	s.Require().NoError(err)
	addr3, err := s.addressCodec.BytesToString(s.addrs[2])
	s.Require().NoError(err)

	addr1st, err := s.addressCodec.BytesToString(s.addrs[0])
	s.Require().NoError(err)

	// Initial group, group policy and balance setup
	members := []group.MemberRequest{
		{Address: addr1st, Weight: "1"}, {Address: addr2, Weight: "2"},
	}

	groupRes, err := s.groupKeeper.CreateGroup(ctx, &group.MsgCreateGroup{
		Admin:   addr1st,
		Members: members,
	})
	s.Require().NoError(err)

	groupRes2, err := s.groupKeeper.CreateGroup(ctx, &group.MsgCreateGroup{
		Admin:   addr2,
		Members: members,
	})
	s.Require().NoError(err)

	groupID := groupRes.GroupId
	groupID2 := groupRes2.GroupId

	policy := group.NewThresholdDecisionPolicy(
		"2",
		time.Second,
		0,
	)

	policyReq := &group.MsgCreateGroupPolicy{
		Admin:   addr1,
		GroupId: groupID,
	}

	err = policyReq.SetDecisionPolicy(policy)
	s.Require().NoError(err)
	policyRes, err := s.groupKeeper.CreateGroupPolicy(ctx, policyReq)
	s.Require().NoError(err)

	policy2 := group.NewThresholdDecisionPolicy(
		"1",
		time.Second,
		0,
	)

	policyReq2 := &group.MsgCreateGroupPolicy{
		Admin:   addr2,
		GroupId: groupID2,
	}

	err = policyReq2.SetDecisionPolicy(policy2)
	s.Require().NoError(err)
	policyRes2, err := s.groupKeeper.CreateGroupPolicy(ctx, policyReq2)
	s.Require().NoError(err)

	groupPolicyAddr, err := s.addressCodec.StringToBytes(policyRes.Address)
	s.Require().NoError(err)
	s.Require().NoError(testutil.FundAccount(ctx, s.bankKeeper, groupPolicyAddr, sdk.Coins{sdk.NewInt64Coin("test", 10000)}))

	groupPolicyAddr2, err := s.addressCodec.StringToBytes(policyRes2.Address)
	s.Require().NoError(err)
	s.Require().NoError(testutil.FundAccount(ctx, s.bankKeeper, groupPolicyAddr2, sdk.Coins{sdk.NewInt64Coin("test", 10000)}))

	votingPeriod := policy.GetVotingPeriod()

	msgSend1 := &banktypes.MsgSend{
		FromAddress: policyRes.Address,
		ToAddress:   addr2,
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}
	msgSend2 := &banktypes.MsgSend{
		FromAddress: policyRes2.Address,
		ToAddress:   addr2,
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}
	proposers := []string{addr2}

	specs := map[string]struct {
		setupProposal     func(ctx sdk.Context) uint64
		expErr            bool
		expErrMsg         string
		newCtx            sdk.Context
		expExecutorResult group.ProposalExecutorResult
		expStatus         group.ProposalStatus
	}{
		"proposal pruned after executor result success": {
			setupProposal: func(ctx sdk.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				pID, err := submitProposalAndVoteHelper(s, s.app, ctx, msgs, proposers, groupPolicyAddr, group.VOTE_OPTION_YES)
				s.Require().NoError(err)
				_, err = s.groupKeeper.Exec(ctx, &group.MsgExec{Executor: addr3, ProposalId: pID})
				s.Require().NoError(err)
				s.Require().NoError(testutil.FundAccount(ctx, s.bankKeeper, groupPolicyAddr, sdk.Coins{sdk.NewInt64Coin("test", 10002)}))

				return pID
			},
			expErrMsg:         "load proposal: not found",
			newCtx:            ctx,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
		},
		"proposal with multiple messages pruned when executed with result success": {
			setupProposal: func(ctx sdk.Context) uint64 {
				msgs := []sdk.Msg{msgSend1, msgSend1}
				pID, err := submitProposalAndVoteHelper(s, s.app, ctx, msgs, proposers, groupPolicyAddr, group.VOTE_OPTION_YES)
				s.Require().NoError(err)
				_, err = s.groupKeeper.Exec(ctx, &group.MsgExec{Executor: addr3, ProposalId: pID})
				s.Require().NoError(err)
				s.Require().NoError(testutil.FundAccount(ctx, s.bankKeeper, groupPolicyAddr, sdk.Coins{sdk.NewInt64Coin("test", 10002)}))

				return pID
			},
			expErrMsg:         "load proposal: not found",
			newCtx:            ctx,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
		},
		"proposal not pruned when not executed and rejected": {
			setupProposal: func(ctx sdk.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				pID, err := submitProposalAndVoteHelper(s, s.app, ctx, msgs, proposers, groupPolicyAddr, group.VOTE_OPTION_NO)
				s.Require().NoError(err)
				_, err = s.groupKeeper.Exec(ctx, &group.MsgExec{Executor: addr3, ProposalId: pID})
				s.Require().NoError(err)

				s.Require().NoError(testutil.FundAccount(ctx, s.bankKeeper, groupPolicyAddr, sdk.Coins{sdk.NewInt64Coin("test", 10002)}))

				return pID
			},
			newCtx:            ctx,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			expStatus:         group.PROPOSAL_STATUS_REJECTED,
		},
		"open proposal is not pruned which must not fail ": {
			setupProposal: func(ctx sdk.Context) uint64 {
				pID, err := submitProposalHelper(s, s.app, ctx, []sdk.Msg{msgSend1}, proposers, groupPolicyAddr)
				s.Require().NoError(err)
				_, err = s.groupKeeper.Exec(ctx, &group.MsgExec{Executor: addr3, ProposalId: pID})
				s.Require().NoError(err)
				s.Require().NoError(testutil.FundAccount(ctx, s.bankKeeper, groupPolicyAddr, sdk.Coins{sdk.NewInt64Coin("test", 10002)}))

				return pID
			},
			newCtx:            ctx,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			expStatus:         group.PROPOSAL_STATUS_SUBMITTED,
		},
		"proposal not pruned with group policy modified before tally": {
			setupProposal: func(ctx sdk.Context) uint64 {
				pID, err := submitProposalHelper(s, s.app, ctx, []sdk.Msg{msgSend1}, proposers, groupPolicyAddr)
				s.Require().NoError(err)
				_, err = s.groupKeeper.UpdateGroupPolicyMetadata(ctx, &group.MsgUpdateGroupPolicyMetadata{
					Admin:              addr1,
					GroupPolicyAddress: policyRes.Address,
				})
				s.Require().NoError(err)
				_, err = s.groupKeeper.Exec(ctx, &group.MsgExec{Executor: addr3, ProposalId: pID})
				s.Require().Error(err) // since proposal with status Aborted cannot be executed
				s.Require().NoError(testutil.FundAccount(ctx, s.bankKeeper, groupPolicyAddr, sdk.Coins{sdk.NewInt64Coin("test", 10002)}))

				return pID
			},
			newCtx:            ctx,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			expStatus:         group.PROPOSAL_STATUS_ABORTED,
		},
		"pruned when proposal is executable when failed before": {
			setupProposal: func(ctx sdk.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				pID, err := submitProposalAndVoteHelper(s, s.app, ctx, msgs, proposers, groupPolicyAddr, group.VOTE_OPTION_YES)
				s.Require().NoError(err)
				_, err = s.groupKeeper.Exec(ctx, &group.MsgExec{Executor: addr3, ProposalId: pID})
				s.Require().NoError(err)
				return pID
			},
			newCtx:            ctx,
			expErrMsg:         "load proposal: not found",
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_SUCCESS,
		},
		"proposal with status withdrawn is pruned after voting period end": {
			setupProposal: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalHelper(s, s.app, sdkCtx, []sdk.Msg{msgSend1}, proposers, groupPolicyAddr)
				s.Require().NoError(err)
				_, err = s.groupKeeper.WithdrawProposal(ctx, &group.MsgWithdrawProposal{
					ProposalId: pID,
					Address:    proposers[0],
				})
				s.Require().NoError(err)
				return pID
			},
			newCtx:    ctx.WithHeaderInfo(header.Info{Time: ctx.HeaderInfo().Time.Add(votingPeriod).Add(time.Hour)}),
			expErrMsg: "load proposal: not found",
			expStatus: group.PROPOSAL_STATUS_WITHDRAWN,
		},
		"proposal with status withdrawn is not pruned (before voting period)": {
			setupProposal: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalHelper(s, s.app, sdkCtx, []sdk.Msg{msgSend1}, proposers, groupPolicyAddr)
				s.Require().NoError(err)
				_, err = s.groupKeeper.WithdrawProposal(ctx, &group.MsgWithdrawProposal{
					ProposalId: pID,
					Address:    proposers[0],
				})
				s.Require().NoError(err)
				return pID
			},
			newCtx:            ctx,
			expErrMsg:         "",
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
			expStatus:         group.PROPOSAL_STATUS_WITHDRAWN,
		},
		"proposal with status aborted is pruned after voting period end (due to updated group policy decision policy)": {
			setupProposal: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalHelper(s, s.app, sdkCtx, []sdk.Msg{msgSend2}, proposers, groupPolicyAddr2)
				s.Require().NoError(err)

				policy := group.NewThresholdDecisionPolicy("3", time.Second, 0)
				msg := &group.MsgUpdateGroupPolicyDecisionPolicy{
					Admin:              addr2,
					GroupPolicyAddress: policyRes2.Address,
				}
				err = msg.SetDecisionPolicy(policy)
				s.Require().NoError(err)
				_, err = s.groupKeeper.UpdateGroupPolicyDecisionPolicy(ctx, msg)
				s.Require().NoError(err)

				return pID
			},
			newCtx:            ctx.WithHeaderInfo(header.Info{Time: ctx.HeaderInfo().Time.Add(votingPeriod).Add(time.Hour)}),
			expErrMsg:         "load proposal: not found",
			expStatus:         group.PROPOSAL_STATUS_ABORTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
		},
		"proposal with status aborted is not pruned before voting period end (due to updated group policy)": {
			setupProposal: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalHelper(s, s.app, sdkCtx, []sdk.Msg{msgSend2}, proposers, groupPolicyAddr2)
				s.Require().NoError(err)

				policy := group.NewThresholdDecisionPolicy("3", time.Second, 0)
				msg := &group.MsgUpdateGroupPolicyDecisionPolicy{
					Admin:              addr2,
					GroupPolicyAddress: policyRes2.Address,
				}
				err = msg.SetDecisionPolicy(policy)
				s.Require().NoError(err)
				_, err = s.groupKeeper.UpdateGroupPolicyDecisionPolicy(ctx, msg)
				s.Require().NoError(err)

				return pID
			},
			newCtx:            ctx,
			expErrMsg:         "",
			expStatus:         group.PROPOSAL_STATUS_ABORTED,
			expExecutorResult: group.PROPOSAL_EXECUTOR_RESULT_NOT_RUN,
		},
	}
	for msg, spec := range specs {
		s.Run(msg, func() {
			proposalID := spec.setupProposal(ctx)

			err := s.groupKeeper.EndBlocker(spec.newCtx)
			s.Require().NoError(err)

			if spec.expErrMsg != "" && spec.expExecutorResult != group.PROPOSAL_EXECUTOR_RESULT_SUCCESS {
				_, err = s.groupKeeper.Proposal(spec.newCtx, &group.QueryProposalRequest{ProposalId: proposalID})
				s.Require().Error(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}
			if spec.expExecutorResult == group.PROPOSAL_EXECUTOR_RESULT_SUCCESS {
				// Make sure proposal is deleted from state
				_, err = s.groupKeeper.Proposal(spec.newCtx, &group.QueryProposalRequest{ProposalId: proposalID})
				s.Require().Contains(err.Error(), spec.expErrMsg)
				res, err := s.groupKeeper.VotesByProposal(ctx, &group.QueryVotesByProposalRequest{ProposalId: proposalID})
				s.Require().NoError(err)
				s.Require().Empty(res.GetVotes())
			} else {
				// Check that proposal and votes exists
				res, err := s.groupKeeper.Proposal(spec.newCtx, &group.QueryProposalRequest{ProposalId: proposalID})
				s.Require().NoError(err)
				_, err = s.groupKeeper.VotesByProposal(ctx, &group.QueryVotesByProposalRequest{ProposalId: res.Proposal.Id})
				s.Require().NoError(err)
				s.Require().Equal("", spec.expErrMsg)

				exp := group.ProposalExecutorResult_name[int32(spec.expExecutorResult)]
				got := group.ProposalExecutorResult_name[int32(res.Proposal.ExecutorResult)]
				s.Assert().Equal(exp, got)

				s.Require().Equal(res.GetProposal().Status, spec.expStatus)
			}
		})
	}
}

func (s *IntegrationTestSuite) TestEndBlockerTallying() {
	app := s.app
	ctx := s.ctx

	addrs := s.addrs
	addr0, err := s.addressCodec.BytesToString(addrs[0])
	s.Require().NoError(err)
	addr1, err := s.addressCodec.BytesToString(addrs[1])
	s.Require().NoError(err)
	addr2, err := s.addressCodec.BytesToString(addrs[2])
	s.Require().NoError(err)
	addr3, err := s.addressCodec.BytesToString(addrs[3])
	s.Require().NoError(err)

	// Initial group, group policy and balance setup
	members := []group.MemberRequest{
		{Address: addr1, Weight: "1"}, {Address: addr2, Weight: "2"},
	}

	groupRes, err := s.groupKeeper.CreateGroup(ctx, &group.MsgCreateGroup{
		Admin:   addr0,
		Members: members,
	})
	s.Require().NoError(err)

	groupID := groupRes.GroupId

	policy := group.NewThresholdDecisionPolicy(
		"2",
		time.Second,
		0,
	)

	policyReq := &group.MsgCreateGroupPolicy{
		Admin:   addr0,
		GroupId: groupID,
	}

	err = policyReq.SetDecisionPolicy(policy)
	s.Require().NoError(err)
	policyRes, err := s.groupKeeper.CreateGroupPolicy(ctx, policyReq)
	s.Require().NoError(err)

	groupPolicyAddr, err := s.addressCodec.StringToBytes(policyRes.Address)
	s.Require().NoError(err)

	votingPeriod := policy.GetVotingPeriod()

	msgSend := &banktypes.MsgSend{
		FromAddress: policyRes.Address,
		ToAddress:   addr3,
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}

	proposers := []string{addr2}

	specs := map[string]struct {
		preRun    func(sdkCtx sdk.Context) uint64
		admin     string
		expErrMsg string
		newCtx    sdk.Context
		tallyRes  group.TallyResult
		expStatus group.ProposalStatus
	}{
		"tally updated after voting period end": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalHelper(s, app, sdkCtx, []sdk.Msg{msgSend}, proposers, groupPolicyAddr)
				s.Require().NoError(err)
				return pID
			},
			admin:     proposers[0],
			newCtx:    ctx.WithHeaderInfo(header.Info{Time: ctx.HeaderInfo().Time.Add(votingPeriod).Add(time.Hour)}),
			tallyRes:  group.DefaultTallyResult(),
			expStatus: group.PROPOSAL_STATUS_REJECTED,
		},
		"tally within voting period": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalHelper(s, app, sdkCtx, []sdk.Msg{msgSend}, proposers, groupPolicyAddr)
				s.Require().NoError(err)

				return pID
			},
			admin:     proposers[0],
			newCtx:    ctx,
			tallyRes:  group.DefaultTallyResult(),
			expStatus: group.PROPOSAL_STATUS_SUBMITTED,
		},
		"tally within voting period(with votes)": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalAndVoteHelper(s, app, ctx, []sdk.Msg{msgSend}, proposers, groupPolicyAddr, group.VOTE_OPTION_YES)
				s.Require().NoError(err)

				return pID
			},
			admin:     proposers[0],
			newCtx:    ctx,
			tallyRes:  group.DefaultTallyResult(),
			expStatus: group.PROPOSAL_STATUS_SUBMITTED,
		},
		"tally after voting period (not passing)": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				// `addrs[1]` has weight 1
				pID, err := submitProposalAndVoteHelper(s, app, ctx, []sdk.Msg{msgSend}, []string{addr1}, groupPolicyAddr, group.VOTE_OPTION_YES)
				s.Require().NoError(err)

				return pID
			},
			admin:  proposers[0],
			newCtx: ctx.WithHeaderInfo(header.Info{Time: ctx.HeaderInfo().Time.Add(votingPeriod).Add(time.Hour)}),
			tallyRes: group.TallyResult{
				YesCount:        "1",
				NoCount:         "0",
				NoWithVetoCount: "0",
				AbstainCount:    "0",
			},
			expStatus: group.PROPOSAL_STATUS_REJECTED,
		},
		"tally after voting period(with votes)": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalAndVoteHelper(s, app, ctx, []sdk.Msg{msgSend}, proposers, groupPolicyAddr, group.VOTE_OPTION_YES)
				s.Require().NoError(err)

				return pID
			},
			admin:  proposers[0],
			newCtx: ctx.WithHeaderInfo(header.Info{Time: ctx.HeaderInfo().Time.Add(votingPeriod).Add(time.Hour)}),
			tallyRes: group.TallyResult{
				YesCount:        "2",
				NoCount:         "0",
				NoWithVetoCount: "0",
				AbstainCount:    "0",
			},
			expStatus: group.PROPOSAL_STATUS_ACCEPTED,
		},
		"tally of withdrawn proposal": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalHelper(s, app, sdkCtx, []sdk.Msg{msgSend}, proposers, groupPolicyAddr)
				s.Require().NoError(err)

				_, err = s.groupKeeper.WithdrawProposal(ctx, &group.MsgWithdrawProposal{
					ProposalId: pID,
					Address:    proposers[0],
				})

				s.Require().NoError(err)
				return pID
			},
			admin:     proposers[0],
			newCtx:    ctx,
			tallyRes:  group.DefaultTallyResult(),
			expStatus: group.PROPOSAL_STATUS_WITHDRAWN,
		},
		"tally of withdrawn proposal (with votes)": {
			preRun: func(sdkCtx sdk.Context) uint64 {
				pID, err := submitProposalAndVoteHelper(s, app, ctx, []sdk.Msg{msgSend}, proposers, groupPolicyAddr, group.VOTE_OPTION_YES)
				s.Require().NoError(err)

				_, err = s.groupKeeper.WithdrawProposal(ctx, &group.MsgWithdrawProposal{
					ProposalId: pID,
					Address:    proposers[0],
				})

				s.Require().NoError(err)
				return pID
			},
			admin:     proposers[0],
			newCtx:    ctx,
			tallyRes:  group.DefaultTallyResult(),
			expStatus: group.PROPOSAL_STATUS_WITHDRAWN,
		},
	}

	for msg, spec := range specs {
		s.Run(msg, func() {
			pID := spec.preRun(ctx)

			err := s.groupKeeper.EndBlocker(spec.newCtx)
			s.Require().NoError(err)
			resp, err := s.groupKeeper.Proposal(spec.newCtx, &group.QueryProposalRequest{
				ProposalId: pID,
			})

			if spec.expErrMsg != "" {
				s.Require().NoError(err)
				s.Require().Contains(err.Error(), spec.expErrMsg)
				return
			}

			s.Require().NoError(err)
			s.Require().Equal(resp.GetProposal().FinalTallyResult, spec.tallyRes)
			s.Require().Equal(resp.GetProposal().Status, spec.expStatus)
		})
	}
}

func submitProposalHelper(s *IntegrationTestSuite, app *runtime.App, ctx context.Context, msgs []sdk.Msg, proposers []string, groupPolicyAddr sdk.AccAddress) (uint64, error) {
	gpAddr, err := s.addressCodec.BytesToString(groupPolicyAddr)
	s.Require().NoError(err)
	proposalReq := &group.MsgSubmitProposal{
		GroupPolicyAddress: gpAddr,
		Proposers:          proposers,
	}
	err = proposalReq.SetMsgs(msgs)
	if err != nil {
		return 0, err
	}

	proposalRes, err := s.groupKeeper.SubmitProposal(ctx, proposalReq)
	if err != nil {
		return 0, err
	}

	return proposalRes.ProposalId, nil
}

func submitProposalAndVoteHelper(
	s *IntegrationTestSuite, app *runtime.App, ctx context.Context, msgs []sdk.Msg,
	proposers []string, groupPolicyAddr sdk.AccAddress, voteOption group.VoteOption,
) (uint64, error) {
	myProposalID, err := submitProposalHelper(s, app, ctx, msgs, proposers, groupPolicyAddr)
	if err != nil {
		return 0, err
	}
	_, err = s.groupKeeper.Vote(ctx, &group.MsgVote{
		ProposalId: myProposalID,
		Voter:      proposers[0],
		Option:     voteOption,
	})
	if err != nil {
		return 0, err
	}
	return myProposalID, nil
}
