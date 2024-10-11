package keeper_test

import (
	gocontext "context"
	"fmt"
	"time"

	"cosmossdk.io/math"
	v3 "cosmossdk.io/x/gov/migrations/v3"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	"github.com/cosmos/cosmos-sdk/codec/address"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

func (suite *KeeperTestSuite) TestGRPCQueryProposal() {
	suite.reset()
	ctx, queryClient, addrs := suite.ctx, suite.queryClient, suite.addrs
	govAcctStr, err := suite.acctKeeper.AddressCodec().BytesToString(govAcct)
	suite.Require().NoError(err)
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
				msgContent, err := v1.NewLegacyContent(testProposal, govAcctStr)
				suite.Require().NoError(err)
				submittedProposal, err := suite.govKeeper.SubmitProposal(ctx, []sdk.Msg{msgContent}, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
				suite.Require().NoError(err)
				suite.Require().NotEmpty(submittedProposal)

				expProposal = submittedProposal
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			proposalRes, err := queryClient.Proposal(gocontext.Background(), req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotEmpty(proposalRes.Proposal.String())
				suite.Require().Equal(proposalRes.Proposal.String(), expProposal.String())
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(proposalRes)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQueryConstitution() {
	suite.reset()
	queryClient := suite.queryClient

	expRes := &v1.QueryConstitutionResponse{Constitution: "constitution"}

	constitution, err := queryClient.Constitution(gocontext.Background(), &v1.QueryConstitutionRequest{})
	suite.Require().NoError(err)
	suite.Require().Equal(expRes, constitution)
}

func (suite *KeeperTestSuite) TestLegacyGRPCQueryProposal() {
	suite.reset()
	ctx, queryClient, addrs := suite.ctx, suite.legacyQueryClient, suite.addrs
	govAcctStr, err := suite.acctKeeper.AddressCodec().BytesToString(govAcct)
	suite.Require().NoError(err)

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
				msgContent, err := v1.NewLegacyContent(testProposal, govAcctStr)
				suite.Require().NoError(err)
				submittedProposal, err := suite.govKeeper.SubmitProposal(ctx, []sdk.Msg{msgContent}, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
				suite.Require().NoError(err)
				suite.Require().NotEmpty(submittedProposal)

				expProposal, err = v3.ConvertToLegacyProposal(submittedProposal)
				suite.Require().NoError(err)
			},
			true,
		},
		{
			"valid request - expedited",
			func() {
				req = &v1beta1.QueryProposalRequest{ProposalId: 2}
				testProposal := v1beta1.NewTextProposal("Proposal", "testing proposal")
				msgContent, err := v1.NewLegacyContent(testProposal, govAcctStr)
				suite.Require().NoError(err)
				submittedProposal, err := suite.govKeeper.SubmitProposal(ctx, []sdk.Msg{msgContent}, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_EXPEDITED)
				suite.Require().NoError(err)
				suite.Require().NotEmpty(submittedProposal)

				expProposal, err = v3.ConvertToLegacyProposal(submittedProposal)
				suite.Require().NoError(err)
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			proposalRes, err := queryClient.Proposal(gocontext.Background(), req)

			if tc.expPass {
				suite.Require().NoError(err)
				suite.Require().NotEmpty(proposalRes.Proposal.String())
				suite.Require().Equal(proposalRes.Proposal.String(), expProposal.String())
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
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
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
					govAddress, err := suite.acctKeeper.AddressCodec().BytesToString(suite.govKeeper.GetGovernanceAccount(suite.ctx).GetAddress())
					suite.Require().NoError(err)
					testProposal := []sdk.Msg{
						v1.NewMsgVote(govAddress, uint64(i), v1.OptionYes, ""),
					}
					proposal, err := suite.govKeeper.SubmitProposal(ctx, testProposal, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
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
			"request 2nd page with limit 3",
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
				deposit := v1.NewDeposit(testProposals[0].Id, addr0Str, depositCoins)
				err := suite.govKeeper.SetDeposit(ctx, deposit)
				suite.Require().NoError(err)

				req = &v1.QueryProposalsRequest{
					Depositor: addr0Str,
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
				err := suite.govKeeper.Proposals.Set(ctx, testProposals[1].Id, *testProposals[1])
				suite.Require().NoError(err)
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, testProposals[1].Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionAbstain), ""))

				req = &v1.QueryProposalsRequest{
					Voter: addr0Str,
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: testProposals[1:2],
				}
			},
			true,
		},
		{
			"request with filter of status voting period",
			func() {
				req = &v1.QueryProposalsRequest{
					ProposalStatus: v1.StatusVotingPeriod,
				}

				var proposals []*v1.Proposal
				for i := 0; i < len(testProposals); i++ {
					if testProposals[i].GetStatus() == v1.StatusVotingPeriod {
						proposals = append(proposals, testProposals[i])
					}
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: proposals,
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

				var proposals []*v1.Proposal
				for i := 0; i < len(testProposals); i++ {
					if testProposals[i].GetStatus() == v1.StatusDepositPeriod {
						proposals = append(proposals, testProposals[i])
					}
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: proposals,
				}
			},
			true,
		},
		{
			"request with filter of status deposit period with limit 2",
			func() {
				req = &v1.QueryProposalsRequest{
					ProposalStatus: v1.StatusDepositPeriod,
					Pagination: &query.PageRequest{
						Limit:      2,
						CountTotal: true,
					},
				}

				var proposals []*v1.Proposal
				for i := 0; i < len(testProposals) && len(proposals) < 2; i++ {
					if testProposals[i].GetStatus() == v1.StatusDepositPeriod {
						proposals = append(proposals, testProposals[i])
					}
				}

				expRes = &v1.QueryProposalsResponse{
					Proposals: proposals,
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			proposals, err := queryClient.Proposals(gocontext.Background(), req)

			if tc.expPass {
				suite.Require().NoError(err)

				suite.Require().Len(proposals.GetProposals(), len(expRes.GetProposals()))
				for i := 0; i < len(proposals.GetProposals()); i++ {
					suite.Require().NoError(err)
					suite.Require().NotEmpty(proposals.GetProposals()[i])
					suite.Require().Equal(expRes.GetProposals()[i].String(), proposals.GetProposals()[i].String())
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
	ctx, queryClient, addrs := suite.ctx, suite.legacyQueryClient, suite.addrs
	govAcctStr, err := suite.acctKeeper.AddressCodec().BytesToString(govAcct)
	suite.Require().NoError(err)

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
				msgContent, err := v1.NewLegacyContent(testProposal, govAcctStr)
				suite.Require().NoError(err)
				submittedProposal, err := suite.govKeeper.SubmitProposal(ctx, []sdk.Msg{msgContent}, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
				suite.Require().NoError(err)
				suite.Require().NotEmpty(submittedProposal)
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			proposalRes, err := queryClient.Proposals(gocontext.Background(), req)

			if tc.expPass {
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
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	addr1Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[1])
	suite.Require().NoError(err)

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
					Voter:      addr0Str,
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
					Voter:      addr0Str,
				}
			},
			false,
		},
		{
			"no votes present",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
				suite.Require().NoError(err)

				req = &v1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addr0Str,
				}

				expRes = &v1.QueryVoteResponse{}
			},
			false,
		},
		{
			"valid request",
			func() {
				proposal.Status = v1.StatusVotingPeriod
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, proposal.Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionAbstain), ""))

				req = &v1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addr0Str,
				}

				expRes = &v1.QueryVoteResponse{Vote: &v1.Vote{ProposalId: proposal.Id, Voter: addr0Str, Options: []*v1.WeightedVoteOption{{Option: v1.OptionAbstain, Weight: math.LegacyMustNewDecFromStr("1.0").String()}}}}
			},
			true,
		},
		{
			"wrong voter id request",
			func() {
				req = &v1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addr1Str,
				}

				expRes = &v1.QueryVoteResponse{}
			},
			false,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			vote, err := queryClient.Vote(gocontext.Background(), req)

			if tc.expPass {
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
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	addr1Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[1])
	suite.Require().NoError(err)
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
					Voter:      addr0Str,
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
					Voter:      addr0Str,
				}
			},
			false,
		},
		{
			"no votes present",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
				suite.Require().NoError(err)

				req = &v1beta1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addr0Str,
				}

				expRes = &v1beta1.QueryVoteResponse{}
			},
			false,
		},
		{
			"valid request",
			func() {
				proposal.Status = v1.StatusVotingPeriod
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)
				suite.Require().NoError(suite.govKeeper.AddVote(ctx, proposal.Id, addrs[0], v1.NewNonSplitVoteOption(v1.OptionAbstain), ""))

				req = &v1beta1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addr0Str,
				}

				expRes = &v1beta1.QueryVoteResponse{Vote: v1beta1.Vote{ProposalId: proposal.Id, Voter: addr0Str, Options: []v1beta1.WeightedVoteOption{{Option: v1beta1.OptionAbstain, Weight: math.LegacyMustNewDecFromStr("1.0")}}}}
			},
			true,
		},
		{
			"wrong voter id request",
			func() {
				req = &v1beta1.QueryVoteRequest{
					ProposalId: proposal.Id,
					Voter:      addr1Str,
				}

				expRes = &v1beta1.QueryVoteResponse{}
			},
			false,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			vote, err := queryClient.Vote(gocontext.Background(), req)

			if tc.expPass {
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

	addrs := simtestutil.AddTestAddrsIncremental(suite.bankKeeper, suite.stakingKeeper, ctx, 2, math.NewInt(30000000))
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	addr1Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[1])
	suite.Require().NoError(err)

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
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
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
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)
				votes = []*v1.Vote{
					{ProposalId: proposal.Id, Voter: addr0Str, Options: v1.NewNonSplitVoteOption(v1.OptionAbstain)},
					{ProposalId: proposal.Id, Voter: addr1Str, Options: v1.NewNonSplitVoteOption(v1.OptionYes)},
				}

				codec := address.NewBech32Codec("cosmos")
				accAddr1, err1 := codec.StringToBytes(votes[0].Voter)
				accAddr2, err2 := codec.StringToBytes(votes[1].Voter)
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

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			votes, err := queryClient.Votes(gocontext.Background(), req)

			if tc.expPass {
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

	addrs := simtestutil.AddTestAddrsIncremental(suite.bankKeeper, suite.stakingKeeper, ctx, 2, math.NewInt(30000000))
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	addr1Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[1])
	suite.Require().NoError(err)

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
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
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
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)

				votes = []v1beta1.Vote{
					{ProposalId: proposal.Id, Voter: addr0Str, Options: v1beta1.NewNonSplitVoteOption(v1beta1.OptionAbstain)},
					{ProposalId: proposal.Id, Voter: addr1Str, Options: v1beta1.NewNonSplitVoteOption(v1beta1.OptionYes)},
				}
				codec := address.NewBech32Codec("cosmos")

				accAddr1, err1 := codec.StringToBytes(votes[0].Voter)
				accAddr2, err2 := codec.StringToBytes(votes[1].Voter)
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

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			votes, err := queryClient.Votes(gocontext.Background(), req)

			if tc.expPass {
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
	testCases := []struct {
		msg     string
		req     v1.QueryParamsRequest
		expPass bool
	}{
		{
			"empty request (valid and returns all params)",
			v1.QueryParamsRequest{},
			true,
		},
		{
			"invalid request (but passes as params type is deprecated)",
			v1.QueryParamsRequest{ParamsType: "wrongPath"},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			params, err := queryClient.Params(gocontext.Background(), &tc.req)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(params)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestGRPCQueryMessagedBasedParams() {
	// create custom message based params for x/gov/MsgUpdateParams
	err := suite.govKeeper.MessageBasedParams.Set(suite.ctx, sdk.MsgTypeURL(&v1.MsgUpdateParams{}), v1.MessageBasedParams{
		VotingPeriod:  func() *time.Duration { t := time.Hour * 24 * 7; return &t }(),
		Quorum:        "0.4",
		Threshold:     "0.5",
		VetoThreshold: "0.66",
	})
	suite.Require().NoError(err)

	defaultGovParams := v1.DefaultParams()

	queryClient := suite.queryClient
	testCases := []struct {
		msg       string
		req       v1.QueryMessageBasedParamsRequest
		expResp   *v1.QueryMessageBasedParamsResponse
		expErrMsg string
	}{
		{
			msg:       "empty request",
			req:       v1.QueryMessageBasedParamsRequest{},
			expErrMsg: "invalid request",
		},
		{
			msg: "valid request (custom msg based params)",
			req: v1.QueryMessageBasedParamsRequest{
				MsgUrl: sdk.MsgTypeURL(&v1.MsgUpdateParams{}),
			},
			expResp: &v1.QueryMessageBasedParamsResponse{
				Params: &v1.MessageBasedParams{
					VotingPeriod:  func() *time.Duration { t := time.Hour * 24 * 7; return &t }(),
					Quorum:        "0.4",
					Threshold:     "0.5",
					VetoThreshold: "0.66",
				},
			},
		},
		{
			msg: "valid request (default msg based params)",
			req: v1.QueryMessageBasedParamsRequest{
				MsgUrl: sdk.MsgTypeURL(&v1.MsgSubmitProposal{}),
			},
			expResp: &v1.QueryMessageBasedParamsResponse{
				Params: &v1.MessageBasedParams{
					VotingPeriod:  defaultGovParams.VotingPeriod,
					Quorum:        defaultGovParams.Quorum,
					Threshold:     defaultGovParams.Threshold,
					VetoThreshold: defaultGovParams.VetoThreshold,
				},
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			params, err := queryClient.MessageBasedParams(suite.ctx, &tc.req)
			if tc.expErrMsg != "" {
				suite.Require().Error(err)
				suite.Require().ErrorContains(err, tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
				suite.Require().Equal(tc.expResp, params)
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

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			params, err := queryClient.Params(gocontext.Background(), req)

			if tc.expPass {
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
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)

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
					Depositor:  addr0Str,
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
					Depositor:  addr0Str,
				}
			},
			false,
		},
		{
			"no deposits proposal",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
				suite.Require().NoError(err)
				suite.Require().NotNil(proposal)

				req = &v1.QueryDepositRequest{
					ProposalId: proposal.Id,
					Depositor:  addr0Str,
				}
			},
			false,
		},
		{
			"valid request",
			func() {
				depositCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 20)))
				deposit := v1.NewDeposit(proposal.Id, addr0Str, depositCoins)
				err := suite.govKeeper.SetDeposit(ctx, deposit)
				suite.Require().NoError(err)

				req = &v1.QueryDepositRequest{
					ProposalId: proposal.Id,
					Depositor:  addr0Str,
				}

				expRes = &v1.QueryDepositResponse{Deposit: &deposit}
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			deposit, err := queryClient.Deposit(gocontext.Background(), req)

			if tc.expPass {
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
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)

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
					Depositor:  addr0Str,
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
					Depositor:  addr0Str,
				}
			},
			false,
		},
		{
			"no deposits proposal",
			func() {
				var err error
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
				suite.Require().NoError(err)
				suite.Require().NotNil(proposal)

				req = &v1beta1.QueryDepositRequest{
					ProposalId: proposal.Id,
					Depositor:  addr0Str,
				}
			},
			false,
		},
		{
			"valid request",
			func() {
				depositCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 20)))
				deposit := v1beta1.NewDeposit(proposal.Id, addr0Str, depositCoins)
				v1deposit := v1.NewDeposit(proposal.Id, addr0Str, depositCoins)
				err := suite.govKeeper.SetDeposit(ctx, v1deposit)
				suite.Require().NoError(err)

				req = &v1beta1.QueryDepositRequest{
					ProposalId: proposal.Id,
					Depositor:  addr0Str,
				}

				expRes = &v1beta1.QueryDepositResponse{Deposit: deposit}
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			deposit, err := queryClient.Deposit(gocontext.Background(), req)

			if tc.expPass {
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
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	addr1Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[1])
	suite.Require().NoError(err)

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
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_EXPEDITED)
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
				deposit1 := v1.NewDeposit(proposal.Id, addr0Str, depositAmount1)
				err := suite.govKeeper.SetDeposit(ctx, deposit1)
				suite.Require().NoError(err)

				depositAmount2 := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 30)))
				deposit2 := v1.NewDeposit(proposal.Id, addr1Str, depositAmount2)
				err = suite.govKeeper.SetDeposit(ctx, deposit2)
				suite.Require().NoError(err)

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

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			deposits, err := queryClient.Deposits(gocontext.Background(), req)

			if tc.expPass {
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
	suite.reset()
	ctx, queryClient, addrs := suite.ctx, suite.legacyQueryClient, suite.addrs
	addr0Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	addr1Str, err := suite.acctKeeper.AddressCodec().BytesToString(addrs[1])
	suite.Require().NoError(err)

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
				proposal, err = suite.govKeeper.SubmitProposal(ctx, TestProposal, "", "title", "summary", addrs[0], v1.ProposalType_PROPOSAL_TYPE_STANDARD)
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
				deposit1 := v1beta1.NewDeposit(proposal.Id, addr0Str, depositAmount1)
				v1deposit1 := v1.NewDeposit(proposal.Id, addr0Str, depositAmount1)
				err := suite.govKeeper.SetDeposit(ctx, v1deposit1)
				suite.Require().NoError(err)

				depositAmount2 := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, suite.stakingKeeper.TokensFromConsensusPower(ctx, 30)))
				deposit2 := v1beta1.NewDeposit(proposal.Id, addr1Str, depositAmount2)
				v1deposit2 := v1.NewDeposit(proposal.Id, addr1Str, depositAmount2)
				err = suite.govKeeper.SetDeposit(ctx, v1deposit2)
				suite.Require().NoError(err)

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

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			deposits, err := queryClient.Deposits(gocontext.Background(), req)

			if tc.expPass {
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
	queryClient := suite.queryClient

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
						YesCount:         "4",
						AbstainCount:     "1",
						NoCount:          "0",
						NoWithVetoCount:  "0",
						OptionOneCount:   "4",
						OptionTwoCount:   "1",
						OptionThreeCount: "0",
						OptionFourCount:  "0",
						SpamCount:        "0",
					},
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)

				req = &v1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1.TallyResult{
					YesCount:         "4",
					AbstainCount:     "1",
					NoCount:          "0",
					NoWithVetoCount:  "0",
					OptionOneCount:   "4",
					OptionTwoCount:   "1",
					OptionThreeCount: "0",
					OptionFourCount:  "0",
					SpamCount:        "0",
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
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)

				req = &v1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1.TallyResult{
					YesCount:         "0",
					AbstainCount:     "0",
					NoCount:          "0",
					NoWithVetoCount:  "0",
					OptionOneCount:   "0",
					OptionTwoCount:   "0",
					OptionThreeCount: "0",
					OptionFourCount:  "0",
					SpamCount:        "0",
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
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)

				req = &v1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1.TallyResult{
					YesCount:         "0",
					AbstainCount:     "0",
					NoCount:          "0",
					NoWithVetoCount:  "0",
					OptionOneCount:   "0",
					OptionTwoCount:   "0",
					OptionThreeCount: "0",
					OptionFourCount:  "0",
					SpamCount:        "0",
				}
			},
			true,
		},
		{
			"proposal status failed",
			func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:     1,
					Status: v1.StatusFailed,
					FinalTallyResult: &v1.TallyResult{
						YesCount:         "4",
						AbstainCount:     "1",
						NoCount:          "0",
						NoWithVetoCount:  "0",
						OptionOneCount:   "4",
						OptionTwoCount:   "1",
						OptionThreeCount: "0",
						OptionFourCount:  "0",
						SpamCount:        "0",
					},
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)

				req = &v1.QueryTallyResultRequest{ProposalId: proposal.Id}

				expTally = &v1.TallyResult{
					YesCount:         "4",
					AbstainCount:     "1",
					NoCount:          "0",
					NoWithVetoCount:  "0",
					OptionOneCount:   "4",
					OptionTwoCount:   "1",
					OptionThreeCount: "0",
					OptionFourCount:  "0",
					SpamCount:        "0",
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			tallyRes, err := queryClient.TallyResult(gocontext.Background(), req)

			if tc.expPass {
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
	queryClient := suite.legacyQueryClient

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
						SpamCount:       "0",
					},
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)

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
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)

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
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)
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
			"proposal status failed",
			func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:     1,
					Status: v1.StatusFailed,
					FinalTallyResult: &v1.TallyResult{
						YesCount:        "4",
						AbstainCount:    "1",
						NoCount:         "0",
						NoWithVetoCount: "0",
						SpamCount:       "0",
					},
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
				}
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)

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
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			tallyRes, err := queryClient.TallyResult(gocontext.Background(), req)

			if tc.expPass {
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

func (suite *KeeperTestSuite) TestProposalVoteOptions() {
	suite.reset()

	testCases := []struct {
		name     string
		malleate func()
		req      *v1.QueryProposalVoteOptionsRequest
		expResp  *v1.QueryProposalVoteOptionsResponse
		errStr   string
	}{
		{
			name:   "invalid proposal id",
			req:    &v1.QueryProposalVoteOptionsRequest{},
			errStr: "proposal id can not be 0",
		},
		{
			name:   "proposal not found",
			req:    &v1.QueryProposalVoteOptionsRequest{ProposalId: 1},
			errStr: "proposal 1 doesn't exist",
		},
		{
			name: "non multiple choice proposal",
			malleate: func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:              1,
					Status:          v1.StatusVotingPeriod,
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
					ProposalType:    v1.ProposalType_PROPOSAL_TYPE_STANDARD,
				}
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)
			},
			req: &v1.QueryProposalVoteOptionsRequest{ProposalId: 1},
			expResp: &v1.QueryProposalVoteOptionsResponse{
				VoteOptions: &v1.ProposalVoteOptions{
					OptionOne:   "yes",
					OptionTwo:   "abstain",
					OptionThree: "no",
					OptionFour:  "no_with_veto",
					OptionSpam:  "spam",
				},
			},
		},
		{
			name: "invalid multiple choice proposal",
			req:  &v1.QueryProposalVoteOptionsRequest{ProposalId: 2},
			malleate: func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:              2,
					Status:          v1.StatusVotingPeriod,
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
					ProposalType:    v1.ProposalType_PROPOSAL_TYPE_MULTIPLE_CHOICE,
				}
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)

				// multiple choice proposal, but no vote options set
				// because the query does not check the proposal type,
				// it falls back to the default vote options
			},
			expResp: &v1.QueryProposalVoteOptionsResponse{
				VoteOptions: &v1.ProposalVoteOptions{
					OptionOne:   "yes",
					OptionTwo:   "abstain",
					OptionThree: "no",
					OptionFour:  "no_with_veto",
					OptionSpam:  "spam",
				},
			},
		},
		{
			name: "multiple choice proposal",
			req:  &v1.QueryProposalVoteOptionsRequest{ProposalId: 3},
			malleate: func() {
				propTime := time.Now()
				proposal := v1.Proposal{
					Id:              3,
					Status:          v1.StatusVotingPeriod,
					SubmitTime:      &propTime,
					VotingStartTime: &propTime,
					VotingEndTime:   &propTime,
					Metadata:        "proposal metadata",
					ProposalType:    v1.ProposalType_PROPOSAL_TYPE_MULTIPLE_CHOICE,
				}
				err := suite.govKeeper.Proposals.Set(suite.ctx, proposal.Id, proposal)
				suite.Require().NoError(err)
				err = suite.govKeeper.ProposalVoteOptions.Set(suite.ctx, proposal.Id, v1.ProposalVoteOptions{
					OptionOne:   "Vote for @tac0turle",
					OptionTwo:   "Vote for @facudomedica",
					OptionThree: "Vote for @alexanderbez",
				})
				suite.Require().NoError(err)
			},
			expResp: &v1.QueryProposalVoteOptionsResponse{
				VoteOptions: &v1.ProposalVoteOptions{
					OptionOne:   "Vote for @tac0turle",
					OptionTwo:   "Vote for @facudomedica",
					OptionThree: "Vote for @alexanderbez",
					OptionSpam:  "spam",
				},
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			if tc.malleate != nil {
				tc.malleate()
			}

			resp, err := suite.queryClient.ProposalVoteOptions(suite.ctx, tc.req)
			if tc.errStr != "" {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.errStr)
			} else {
				suite.Require().NoError(err)
				suite.Require().Equal(tc.expResp, resp)
			}
		})
	}
}
