package keeper_test

import (
	gocontext "context"
	"fmt"
	"time"

	"cosmossdk.io/math"

	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
	v3 "github.com/cosmos/cosmos-sdk/x/gov/migrations/v3"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

func (suite *KeeperTestSuite) TestGRPCQueryProposal() {
	suite.reset()
	ctx, queryClient := suite.ctx, suite.queryClient

	var (
		req         *v1.QueryProposalRequest
		expProposal v1.Proposal
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1.QueryProposalRequest{}
			},
			false,
		},
		{
			"non existing proposal request",
			func() {
				req = &v1.QueryProposalRequest{ProposalId: 2}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1.QueryProposalRequest{ProposalId: 0}
			},
			false,
		},
		{
			"valid request",
			func() {
				req = &v1.QueryProposalRequest{ProposalId: 1}
				testProposal := v1beta1.NewTextProposal("Proposal", "testing proposal")
				msgContent, err := v1.NewLegacyContent(testProposal, govAcct.String())
				suite.Require().NoError(err)
				submittedProposal, err := suite.govKeeper.SubmitProposal(ctx, []sdk.Msg{msgContent}, "", "test", "summary", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
				suite.Require().NoError(err)
				suite.Require().NotEmpty(submittedProposal)

				expProposal = submittedProposal
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			proposalRes, err := queryClient.Proposal(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				// Instead of using MashalJSON, we could compare .String() output too.
				// https://github.com/cosmos/cosmos-sdk/issues/10965
				expJSON, err := suite.cdc.MarshalJSON(&expProposal)
				suite.Require().NoError(err)
				actualJSON, err := suite.cdc.MarshalJSON(proposalRes.Proposal)
				suite.Require().NoError(err)
				suite.Require().Equal(expJSON, actualJSON)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(proposalRes)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyGRPCQueryProposal() {
	suite.reset()
	ctx, queryClient := suite.ctx, suite.legacyQueryClient

	var (
		req         *v1beta1.QueryProposalRequest
		expProposal v1beta1.Proposal
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1beta1.QueryProposalRequest{}
			},
			false,
		},
		{
			"non existing proposal request",
			func() {
				req = &v1beta1.QueryProposalRequest{ProposalId: 3}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1beta1.QueryProposalRequest{ProposalId: 0}
			},
			false,
		},
		{
			"valid request",
			func() {
				req = &v1beta1.QueryProposalRequest{ProposalId: 1}
				testProposal := v1beta1.NewTextProposal("Proposal", "testing proposal")
				msgContent, err := v1.NewLegacyContent(testProposal, govAcct.String())
				suite.Require().NoError(err)
				submittedProposal, err := suite.govKeeper.SubmitProposal(ctx, []sdk.Msg{msgContent}, "", "test", "summary", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
				suite.Require().NoError(err)
				suite.Require().NotEmpty(submittedProposal)

				expProposal, err = v3.ConvertToLegacyProposal(submittedProposal)
				suite.Require().NoError(err)
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			proposalRes, err := queryClient.Proposal(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				// Instead of using MashalJSON, we could compare .String() output too.
				// https://github.com/cosmos/cosmos-sdk/issues/10965
				expJSON, err := suite.cdc.MarshalJSON(&expProposal)
				suite.Require().NoError(err)
				actualJSON, err := suite.cdc.MarshalJSON(&proposalRes.Proposal)
				suite.Require().NoError(err)
				suite.Require().Equal(expJSON, actualJSON)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(proposalRes)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQueryProposals() {
	suite.reset()
	ctx, queryClient, addrs := suite.ctx, suite.queryClient, suite.addrs

	testProposals := []*v1.Proposal{}

	var (
		req    *v1.QueryProposalsRequest
		expRes *v1.QueryProposalsResponse
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty state request",
			func() {
				req = &v1.QueryProposalsRequest{}
			},
			true,
		},
		{
			"request proposals with limit 3",
			func() {
				// create 5 test proposals
				for i := 0; i < 5; i++ {
					govAddress := suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress()
					testProposal := []sdk.Msg{
						v1.NewMsgVote(govAddress, uint64(i), v1.OptionYes, ""),
					}
					proposal, err := suite.govKeeper.SubmitProposal(ctx, testProposal, "", "test", "summary", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
					suite.Require().NotEmpty(proposal)
					suite.Require().NoError(err)
					testProposals = append(testProposals, &proposal)
				}

				req = &v1.QueryProposalsRequest{
					Pagination: &query.PageRequest{Limit: 3},
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: testProposals[:3],
				}
			},
			true,
		},
		{
			"request 2nd page with limit 4",
			func() {
				req = &v1.QueryProposalsRequest{
					Pagination: &query.PageRequest{Offset: 3, Limit: 3},
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: testProposals[3:],
				}
			},
			true,
		},
		{
			"request with limit 2 and count true",
			func() {
				req = &v1.QueryProposalsRequest{
					Pagination: &query.PageRequest{Limit: 2, CountTotal: true},
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: testProposals[:2],
				}
			},
			true,
		},
		{
			"request with filter of status deposit period",
			func() {
				req = &v1.QueryProposalsRequest{
					ProposalStatus: v1.StatusDepositPeriod,
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: testProposals,
				}
			},
			true,
		},
		{
			"request with filter of deposit address",
			func() {
				depositCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 20)))
				deposit := v1.NewDeposit(testProposals[0].Id, addrs[0], depositCoins)
				suite.govKeeper.SetDeposit(ctx, deposit)

				req = &v1.QueryProposalsRequest{
					Depositor: addrs[0].String(),
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: testProposals[:1],
				}
			},
			true,
		},
		{
			"request with filter of deposit address",
			func() {
				testProposals[1].Status = v1.StatusVotingPeriod
				suite.govKeeper.SetProposal(ctx, *testProposals[1])
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, testProposals[1].Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionAbstain), ""))

				req = &v1.QueryProposalsRequest{
					Voter: addrs[0].String(),
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: testProposals[1:2],
				}
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			proposals, err := queryClient.Proposals(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)

				suite.Require().Len(proposals.GetProposals(), len(expRes.GetProposals()))
				for i := 0; i < len(proposals.GetProposals()); i++ {
					// Instead of using MashalJSON, we could compare .String() output too.
					// https://github.com/cosmos/cosmos-sdk/issues/10965
					expJSON, err := suite.cdc.MarshalJSON(expRes.GetProposals()[i])
					suite.Require().NoError(err)
					actualJSON, err := suite.cdc.MarshalJSON(proposals.GetProposals()[i])
					suite.Require().NoError(err)

					suite.Require().Equal(expJSON, actualJSON)
				}

			} else {
				suite.Require().Error(err)
				suite.Require().Nil(proposals)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyGRPCQueryProposals() {
	suite.reset()
	ctx, queryClient := suite.ctx, suite.legacyQueryClient

	var req *v1beta1.QueryProposalsRequest

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"valid request",
			func() {
				req = &v1beta1.QueryProposalsRequest{}
				testProposal := v1beta1.NewTextProposal("Proposal", "testing proposal")
				msgContent, err := v1.NewLegacyContent(testProposal, govAcct.String())
				suite.Require().NoError(err)
				submittedProposal, err := suite.govKeeper.SubmitProposal(ctx, []sdk.Msg{msgContent}, "", "test", "summary", sdk.AccAddress("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r"))
				suite.Require().NoError(err)
				suite.Require().NotEmpty(submittedProposal)
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			proposalRes, err := queryClient.Proposals(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(proposalRes.Proposals)
				suite.Require().Equal(len(proposalRes.Proposals), 1)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(proposalRes)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQueryVote() {
	ctx, queryClient, addrs := suite.ctx, suite.queryClient, suite.addrs

	var (
		req      *v1.QueryVoteRequest
		expRes   *v1.QueryVoteResponse
		proposal v1.Proposal
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1.QueryVoteRequest{}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1.QueryVoteRequest{
					ProposalId: 0,
					Voter:      addrs[0].String(),
				}
			},
			false,
		},
		{
			"empty voter request",
			func() {
				req = &v1.QueryVoteRequest{
					ProposalId: 1,
					Voter:      "",
				}
			},
			false,
		},
		{
			"non existed proposal",
			func() {
				req = &v1.QueryVoteRequest{
					ProposalId: 3,
					Voter:      addrs[0].String(),
				}
			},
			false,
		},
		{
			"no votes present",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "test", "summary", addrs[0])
				suite.Require().NoError(err)

				req = &v1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addrs[0].String(),
				}

				expRes = &v1.QueryVoteResponse{}
			},
			false,
		},
		{
			"valid request",
			func() {
				proposal.Status = v1.StatusVotingPeriod
				suite.govKeeper.SetProposal(ctx, proposal)
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, proposal.Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionAbstain), ""))

				req = &v1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addrs[0].String(),
				}

				expRes = &v1.QueryVoteResponse{Vote: &v1.Vote{ProposalId: proposal.Id, Voter: addrs[0].String(), Options: []*v1.WeightedVoteOption{{Option: v1.OptionAbstain, Weight: sdk.MustNewDecFromStr("1.0").String()}}}}
			},
			true,
		},
		{
			"wrong voter id request",
			func() {
				req = &v1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addrs[1].String(),
				}

				expRes = &v1.QueryVoteResponse{}
			},
			false,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			vote, err := queryClient.Vote(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expRes, vote)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(vote)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyGRPCQueryVote() {
	ctx, queryClient, addrs := suite.ctx, suite.legacyQueryClient, suite.addrs

	var (
		req      *v1beta1.QueryVoteRequest
		expRes   *v1beta1.QueryVoteResponse
		proposal v1.Proposal
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1beta1.QueryVoteRequest{}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1beta1.QueryVoteRequest{
					ProposalId: 0,
					Voter:      addrs[0].String(),
				}
			},
			false,
		},
		{
			"empty voter request",
			func() {
				req = &v1beta1.QueryVoteRequest{
					ProposalId: 1,
					Voter:      "",
				}
			},
			false,
		},
		{
			"non existed proposal",
			func() {
				req = &v1beta1.QueryVoteRequest{
					ProposalId: 3,
					Voter:      addrs[0].String(),
				}
			},
			false,
		},
		{
			"no votes present",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "test", "summary", addrs[0])
				suite.Require().NoError(err)

				req = &v1beta1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addrs[0].String(),
				}

				expRes = &v1beta1.QueryVoteResponse{}
			},
			false,
		},
		{
			"valid request",
			func() {
				proposal.Status = v1.StatusVotingPeriod
				suite.govKeeper.SetProposal(ctx, proposal)
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, proposal.Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionAbstain), ""))

				req = &v1beta1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addrs[0].String(),
				}

				expRes = &v1beta1.QueryVoteResponse{Vote: v1beta1.Vote{ProposalId: proposal.Id, Voter: addrs[0].String(), Options: []v1beta1.WeightedVoteOption{{Option: v1beta1.OptionAbstain, Weight: sdk.MustNewDecFromStr("1.0")}}}}
			},
			true,
		},
		{
			"wrong voter id request",
			func() {
				req = &v1beta1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addrs[1].String(),
				}

				expRes = &v1beta1.QueryVoteResponse{}
			},
			false,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			vote, err := queryClient.Vote(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expRes, vote)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(vote)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQueryVotes() {
	suite.reset()
	ctx, queryClient := suite.ctx, suite.queryClient

	addrs := simtestutil.AddTestAddrsIncremental(suite.bankKeeper, suite.stakingKeeper, ctx, 2, sdk.NewInt(30000000))

	var (
		req      *v1.QueryVotesRequest
		expRes   *v1.QueryVotesResponse
		proposal v1.Proposal
		votes    v1.Votes
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1.QueryVotesRequest{}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1.QueryVotesRequest{
					ProposalId: 0,
				}
			},
			false,
		},
		{
			"non existed proposals",
			func() {
				req = &v1.QueryVotesRequest{
					ProposalId: 2,
				}
			},
			true,
		},
		{
			"create a proposal and get votes",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "test", "summary", addrs[0])
				suite.Require().NoError(err)

				req = &v1.QueryVotesRequest{
					ProposalId: proposal.Id,
				}
			},
			true,
		},
		{
			"request after adding 2 votes",
			func() {
				proposal.Status = v1.StatusVotingPeriod
				suite.govKeeper.SetProposal(ctx, proposal)

				votes = []*v1.Vote{
					{ProposalId: proposal.Id, Voter: addrs[0].String(), Options: v1.NewNonSplitVoteOption(v1.OptionAbstain)},
					{ProposalId: proposal.Id, Voter: addrs[1].String(), Options: v1.NewNonSplitVoteOption(v1.OptionYes)},
				}
				accAddr1, err1 := sdk.AccAddressFromBech32(votes[0].Voter)
				accAddr2, err2 := sdk.AccAddressFromBech32(votes[1].Voter)
				suite.Require().NoError(err1)
				suite.Require().NoError(err2)
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, proposal.Id, accAddr1, votes[0].Options, ""))
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, proposal.Id, accAddr2, votes[1].Options, ""))

				req = &v1.QueryVotesRequest{
					ProposalId: proposal.Id,
				}

				expRes = &v1.QueryVotesResponse{
					Votes: votes,
				}
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			votes, err := queryClient.Votes(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expRes.GetVotes(), votes.GetVotes())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(votes)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyGRPCQueryVotes() {
	suite.reset()
	ctx, queryClient := suite.ctx, suite.legacyQueryClient

	addrs := simtestutil.AddTestAddrsIncremental(suite.bankKeeper, suite.stakingKeeper, ctx, 2, sdk.NewInt(30000000))

	var (
		req      *v1beta1.QueryVotesRequest
		expRes   *v1beta1.QueryVotesResponse
		proposal v1.Proposal
		votes    v1beta1.Votes
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1beta1.QueryVotesRequest{}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1beta1.QueryVotesRequest{
					ProposalId: 0,
				}
			},
			false,
		},
		{
			"non existed proposals",
			func() {
				req = &v1beta1.QueryVotesRequest{
					ProposalId: 2,
				}
			},
			true,
		},
		{
			"create a proposal and get votes",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "test", "summary", addrs[0])
				suite.Require().NoError(err)

				req = &v1beta1.QueryVotesRequest{
					ProposalId: proposal.Id,
				}
			},
			true,
		},
		{
			"request after adding 2 votes",
			func() {
				proposal.Status = v1.StatusVotingPeriod
				suite.govKeeper.SetProposal(ctx, proposal)

				votes = []v1beta1.Vote{
					{ProposalId: proposal.Id, Voter: addrs[0].String(), Options: v1beta1.NewNonSplitVoteOption(v1beta1.OptionAbstain)},
					{ProposalId: proposal.Id, Voter: addrs[1].String(), Options: v1beta1.NewNonSplitVoteOption(v1beta1.OptionYes)},
				}
				accAddr1, err1 := sdk.AccAddressFromBech32(votes[0].Voter)
				accAddr2, err2 := sdk.AccAddressFromBech32(votes[1].Voter)
				suite.Require().NoError(err1)
				suite.Require().NoError(err2)
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, proposal.Id, accAddr1, v1.NewNonSplitVoteOption(v1.OptionAbstain), ""))
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, proposal.Id, accAddr2, v1.NewNonSplitVoteOption(v1.OptionYes), ""))

				req = &v1beta1.QueryVotesRequest{
					ProposalId: proposal.Id,
				}

				expRes = &v1beta1.QueryVotesResponse{
					Votes: votes,
				}
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			votes, err := queryClient.Votes(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expRes.GetVotes(), votes.GetVotes())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(votes)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQueryParams() {
	queryClient := suite.queryClient

	params := v1.DefaultParams()

	var (
		req    *v1.QueryParamsRequest
		expRes *v1.QueryParamsResponse
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1.QueryParamsRequest{}
			},
			false,
		},
		{
			"deposit params request",
			func() {
				req = &v1.QueryParamsRequest{ParamsType: v1.ParamDeposit}
				depositParams := v1.NewDepositParams(params.MinDeposit, params.MaxDepositPeriod)
				expRes = &v1.QueryParamsResponse{
					DepositParams: &depositParams,
				}
			},
			true,
		},
		{
			"voting params request",
			func() {
				req = &v1.QueryParamsRequest{ParamsType: v1.ParamVoting}
				votingParams := v1.NewVotingParams(params.VotingPeriod)
				expRes = &v1.QueryParamsResponse{
					VotingParams: &votingParams,
				}
			},
			true,
		},
		{
			"tally params request",
			func() {
				req = &v1.QueryParamsRequest{ParamsType: v1.ParamTallying}
				tallyParams := v1.NewTallyParams(params.Quorum, params.Threshold, params.VetoThreshold)
				expRes = &v1.QueryParamsResponse{
					TallyParams: &tallyParams,
				}
			},
			true,
		},
		{
			"invalid request",
			func() {
				req = &v1.QueryParamsRequest{ParamsType: "wrongPath"}
				expRes = &v1.QueryParamsResponse{}
			},
			false,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			params, err := queryClient.Params(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expRes.GetDepositParams(), params.GetDepositParams())
				suite.Require().Equal(expRes.GetVotingParams(), params.GetVotingParams())
				suite.Require().Equal(expRes.GetTallyParams(), params.GetTallyParams())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(params)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyGRPCQueryParams() {
	queryClient := suite.legacyQueryClient

	var (
		req    *v1beta1.QueryParamsRequest
		expRes *v1beta1.QueryParamsResponse
	)

	defaultTallyParams := v1beta1.TallyParams{
		Quorum:        math.LegacyNewDec(0),
		Threshold:     math.LegacyNewDec(0),
		VetoThreshold: math.LegacyNewDec(0),
	}

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1beta1.QueryParamsRequest{}
			},
			false,
		},
		{
			"deposit params request",
			func() {
				req = &v1beta1.QueryParamsRequest{ParamsType: v1beta1.ParamDeposit}
				depositParams := v1beta1.DefaultDepositParams()
				expRes = &v1beta1.QueryParamsResponse{
					DepositParams: depositParams,
					TallyParams:   defaultTallyParams,
				}
			},
			true,
		},
		{
			"voting params request",
			func() {
				req = &v1beta1.QueryParamsRequest{ParamsType: v1beta1.ParamVoting}
				votingParams := v1beta1.DefaultVotingParams()
				expRes = &v1beta1.QueryParamsResponse{
					VotingParams: votingParams,
					TallyParams:  defaultTallyParams,
				}
			},
			true,
		},
		{
			"tally params request",
			func() {
				req = &v1beta1.QueryParamsRequest{ParamsType: v1beta1.ParamTallying}
				tallyParams := v1beta1.DefaultTallyParams()
				expRes = &v1beta1.QueryParamsResponse{
					TallyParams: tallyParams,
				}
			},
			true,
		},
		{
			"invalid request",
			func() {
				req = &v1beta1.QueryParamsRequest{ParamsType: "wrongPath"}
				expRes = &v1beta1.QueryParamsResponse{}
			},
			false,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			params, err := queryClient.Params(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expRes.GetDepositParams(), params.GetDepositParams())
				suite.Require().Equal(expRes.GetVotingParams(), params.GetVotingParams())
				suite.Require().Equal(expRes.GetTallyParams(), params.GetTallyParams())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(params)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQueryDeposit() {
	suite.reset()
	ctx, queryClient, addrs := suite.ctx, suite.queryClient, suite.addrs

	var (
		req      *v1.QueryDepositRequest
		expRes   *v1.QueryDepositResponse
		proposal v1.Proposal
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1.QueryDepositRequest{}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1.QueryDepositRequest{
					ProposalId: 0,
					Depositor:  addrs[0].String(),
				}
			},
			false,
		},
		{
			"empty deposit address request",
			func() {
				req = &v1.QueryDepositRequest{
					ProposalId: 1,
					Depositor:  "",
				}
			},
			false,
		},
		{
			"non existed proposal",
			func() {
				req = &v1.QueryDepositRequest{
					ProposalId: 2,
					Depositor:  addrs[0].String(),
				}
			},
			false,
		},
		{
			"no deposits proposal",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "test", "summary", addrs[0])
				suite.Require().NoError(err)
				suite.Require().NotNil(proposal)

				req = &v1.QueryDepositRequest{
					ProposalId: proposal.Id,
					Depositor:  addrs[0].String(),
				}
			},
			false,
		},
		{
			"valid request",
			func() {
				depositCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 20)))
				deposit := v1.NewDeposit(proposal.Id, addrs[0], depositCoins)
				suite.govKeeper.SetDeposit(ctx, deposit)

				req = &v1.QueryDepositRequest{
					ProposalId: proposal.Id,
					Depositor:  addrs[0].String(),
				}

				expRes = &v1.QueryDepositResponse{Deposit: &deposit}
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			deposit, err := queryClient.Deposit(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(deposit.GetDeposit(), expRes.GetDeposit())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(expRes)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyGRPCQueryDeposit() {
	ctx, queryClient, addrs := suite.ctx, suite.legacyQueryClient, suite.addrs

	var (
		req      *v1beta1.QueryDepositRequest
		expRes   *v1beta1.QueryDepositResponse
		proposal v1.Proposal
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1beta1.QueryDepositRequest{}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1beta1.QueryDepositRequest{
					ProposalId: 0,
					Depositor:  addrs[0].String(),
				}
			},
			false,
		},
		{
			"empty deposit address request",
			func() {
				req = &v1beta1.QueryDepositRequest{
					ProposalId: 1,
					Depositor:  "",
				}
			},
			false,
		},
		{
			"non existed proposal",
			func() {
				req = &v1beta1.QueryDepositRequest{
					ProposalId: 2,
					Depositor:  addrs[0].String(),
				}
			},
			false,
		},
		{
			"no deposits proposal",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "test", "summary", addrs[0])
				suite.Require().NoError(err)
				suite.Require().NotNil(proposal)

				req = &v1beta1.QueryDepositRequest{
					ProposalId: proposal.Id,
					Depositor:  addrs[0].String(),
				}
			},
			false,
		},
		{
			"valid request",
			func() {
				depositCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 20)))
				deposit := v1beta1.NewDeposit(proposal.Id, addrs[0], depositCoins)
				v1deposit := v1.NewDeposit(proposal.Id, addrs[0], depositCoins)
				suite.govKeeper.SetDeposit(ctx, v1deposit)

				req = &v1beta1.QueryDepositRequest{
					ProposalId: proposal.Id,
					Depositor:  addrs[0].String(),
				}

				expRes = &v1beta1.QueryDepositResponse{Deposit: deposit}
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			deposit, err := queryClient.Deposit(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(deposit.GetDeposit(), expRes.GetDeposit())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(expRes)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQueryDeposits() {
	ctx, queryClient, addrs := suite.ctx, suite.queryClient, suite.addrs

	var (
		req      *v1.QueryDepositsRequest
		expRes   *v1.QueryDepositsResponse
		proposal v1.Proposal
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1.QueryDepositsRequest{}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1.QueryDepositsRequest{
					ProposalId: 0,
				}
			},
			false,
		},
		{
			"non existed proposal",
			func() {
				req = &v1.QueryDepositsRequest{
					ProposalId: 2,
				}
			},
			true,
		},
		{
			"create a proposal and get deposits",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "test", "summary", addrs[0])
				suite.Require().NoError(err)

				req = &v1.QueryDepositsRequest{
					ProposalId: proposal.Id,
				}
			},
			true,
		},
		{
			"get deposits with default limit",
			func() {
				depositAmount1 := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 20)))
				deposit1 := v1.NewDeposit(proposal.Id, addrs[0], depositAmount1)
				suite.govKeeper.SetDeposit(ctx, deposit1)

				depositAmount2 := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 30)))
				deposit2 := v1.NewDeposit(proposal.Id, addrs[1], depositAmount2)
				suite.govKeeper.SetDeposit(ctx, deposit2)

				deposits := v1.Deposits{&deposit1, &deposit2}

				req = &v1.QueryDepositsRequest{
					ProposalId: proposal.Id,
				}

				expRes = &v1.QueryDepositsResponse{
					Deposits: deposits,
				}
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			deposits, err := queryClient.Deposits(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expRes.GetDeposits(), deposits.GetDeposits())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(deposits)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyGRPCQueryDeposits() {
	ctx, queryClient, addrs := suite.ctx, suite.legacyQueryClient, suite.addrs

	var (
		req      *v1beta1.QueryDepositsRequest
		expRes   *v1beta1.QueryDepositsResponse
		proposal v1.Proposal
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1beta1.QueryDepositsRequest{}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1beta1.QueryDepositsRequest{
					ProposalId: 0,
				}
			},
			false,
		},
		{
			"non existed proposal",
			func() {
				req = &v1beta1.QueryDepositsRequest{
					ProposalId: 2,
				}
			},
			true,
		},
		{
			"create a proposal and get deposits",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "test", "summary", addrs[0])
				suite.Require().NoError(err)

				req = &v1beta1.QueryDepositsRequest{
					ProposalId: proposal.Id,
				}
			},
			true,
		},
		{
			"get deposits with default limit",
			func() {
				depositAmount1 := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 20)))
				deposit1 := v1beta1.NewDeposit(proposal.Id, addrs[0], depositAmount1)
				v1deposit1 := v1.NewDeposit(proposal.Id, addrs[0], depositAmount1)
				suite.govKeeper.SetDeposit(ctx, v1deposit1)

				depositAmount2 := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 30)))
				deposit2 := v1beta1.NewDeposit(proposal.Id, addrs[1], depositAmount2)
				v1deposit2 := v1.NewDeposit(proposal.Id, addrs[1], depositAmount2)
				suite.govKeeper.SetDeposit(ctx, v1deposit2)

				deposits := v1beta1.Deposits{deposit1, deposit2}

				req = &v1beta1.QueryDepositsRequest{
					ProposalId: proposal.Id,
				}

				expRes = &v1beta1.QueryDepositsResponse{
					Deposits: deposits,
				}
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			deposits, err := queryClient.Deposits(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expRes.GetDeposits(), deposits.GetDeposits())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(deposits)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQueryTallyResult() {
	suite.reset()
	ctx, queryClient := suite.ctx, suite.queryClient

	var (
		req      *v1.QueryTallyResultRequest
		expTally *v1.TallyResult
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1.QueryTallyResultRequest{}
			},
			false,
		},
		{
			"non existing proposal request",
			func() {
				req = &v1.QueryTallyResultRequest{ProposalId: 2}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1.QueryTallyResultRequest{ProposalId: 0}
			},
			false,
		},
		{
			"valid request with proposal status passed",
			func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:     1,
					Status: v1.StatusPassed,
					FinalTallyResult: &v1.TallyResult{
						YesCount:        "4",
						AbstainCount:    "1",
						NoCount:         "0",
						NoWithVetoCount: "0",
					},
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				suite.govKeeper.SetProposal(ctx, proposal)

				req = &v1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1.TallyResult{
					YesCount:        "4",
					AbstainCount:    "1",
					NoCount:         "0",
					NoWithVetoCount: "0",
				}
			},
			true,
		},
		{
			"proposal status deposit",
			func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:              1,
					Status:          v1.StatusDepositPeriod,
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				suite.govKeeper.SetProposal(ctx, proposal)

				req = &v1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1.TallyResult{
					YesCount:        "0",
					AbstainCount:    "0",
					NoCount:         "0",
					NoWithVetoCount: "0",
				}
			},
			true,
		},
		{
			"proposal is in voting period",
			func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:              1,
					Status:          v1.StatusVotingPeriod,
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				suite.govKeeper.SetProposal(ctx, proposal)

				req = &v1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1.TallyResult{
					YesCount:        "0",
					AbstainCount:    "0",
					NoCount:         "0",
					NoWithVetoCount: "0",
				}
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			tallyRes, err := queryClient.TallyResult(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().NotEmpty(tallyRes.Tally.String())
				suite.Require().Equal(expTally.String(), tallyRes.Tally.String())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(tallyRes)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestLegacyGRPCQueryTallyResult() {
	suite.reset()
	ctx, queryClient := suite.ctx, suite.legacyQueryClient

	var (
		req      *v1beta1.QueryTallyResultRequest
		expTally *v1beta1.TallyResult
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"empty request",
			func() {
				req = &v1beta1.QueryTallyResultRequest{}
			},
			false,
		},
		{
			"non existing proposal request",
			func() {
				req = &v1beta1.QueryTallyResultRequest{ProposalId: 2}
			},
			false,
		},
		{
			"zero proposal id request",
			func() {
				req = &v1beta1.QueryTallyResultRequest{ProposalId: 0}
			},
			false,
		},
		{
			"valid request with proposal status passed",
			func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:     1,
					Status: v1.StatusPassed,
					FinalTallyResult: &v1.TallyResult{
						YesCount:        "4",
						AbstainCount:    "1",
						NoCount:         "0",
						NoWithVetoCount: "0",
					},
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				suite.govKeeper.SetProposal(ctx, proposal)

				req = &v1beta1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1beta1.TallyResult{
					Yes:        math.NewInt(4),
					Abstain:    math.NewInt(1),
					No:         math.NewInt(0),
					NoWithVeto: math.NewInt(0),
				}
			},
			true,
		},
		{
			"proposal status deposit",
			func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:              1,
					Status:          v1.StatusDepositPeriod,
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				suite.govKeeper.SetProposal(ctx, proposal)

				req = &v1beta1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1beta1.TallyResult{
					Yes:        math.NewInt(0),
					Abstain:    math.NewInt(0),
					No:         math.NewInt(0),
					NoWithVeto: math.NewInt(0),
				}
			},
			true,
		},
		{
			"proposal is in voting period",
			func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:              1,
					Status:          v1.StatusVotingPeriod,
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				suite.govKeeper.SetProposal(ctx, proposal)

				req = &v1beta1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1beta1.TallyResult{
					Yes:        math.NewInt(0),
					Abstain:    math.NewInt(0),
					No:         math.NewInt(0),
					NoWithVeto: math.NewInt(0),
				}
			},
			true,
		},
	}

	for _, testCase := range testCases {
		suite.Run(fmt.Sprintf("Case %s", testCase.msg), func() {
			testCase.malleate()

			tallyRes, err := queryClient.TallyResult(gocontext.Background(), req)

			if testCase.expPass {
				suite.Require().NoError(err)
				suite.Require().NotEmpty(tallyRes.Tally.String())
				suite.Require().Equal(expTally.String(), tallyRes.Tally.String())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(tallyRes)
			}
		})
	}
}
