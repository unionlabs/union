package keeper_test

import (
	"context"
	"time"

	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/group"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *TestSuite) TestTally() {
	msgSend1 := &banktypes.MsgSend{
		FromAddress: s.groupPolicyStrAddr,
		ToAddress:   s.addrsStr[1],
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}
	proposers := []string{s.addrsStr[1]}

	specs := map[string]struct {
		srcBlockTime   time.Time
		setupProposal  func(ctx context.Context) uint64
		expErr         bool
		expTallyResult group.TallyResult
	}{
		"invalid proposal id": {
			setupProposal: func(ctx context.Context) uint64 {
				return 123
			},
			expErr: true,
		},
		"proposal with no votes": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				return submitProposal(ctx, s, msgs, proposers)
			},
			expTallyResult: group.DefaultTallyResult(),
		},
		"withdrawn proposal": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				proposalID := submitProposal(ctx, s, msgs, proposers)
				_, err := s.groupKeeper.WithdrawProposal(ctx, &group.MsgWithdrawProposal{
					ProposalId: proposalID,
					Address:    proposers[0],
				})
				s.Require().NoError(err)

				return proposalID
			},
			expErr: true,
		},
		"proposal with some votes": {
			setupProposal: func(ctx context.Context) uint64 {
				msgs := []sdk.Msg{msgSend1}
				return submitProposalAndVote(ctx, s, msgs, proposers, group.VOTE_OPTION_YES)
			},
			expTallyResult: group.TallyResult{
				YesCount:        "2",
				NoCount:         "0",
				NoWithVetoCount: "0",
				AbstainCount:    "0",
			},
		},
	}

	for msg, spec := range specs {
		s.Run(msg, func() {
			sdkCtx, _ := s.sdkCtx.CacheContext()
			pID := spec.setupProposal(sdkCtx)
			req := &group.QueryTallyResultRequest{
				ProposalId: pID,
			}

			res, err := s.groupKeeper.TallyResult(sdkCtx, req)
			if spec.expErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
				s.Require().Equal(res.Tally, spec.expTallyResult)
			}
		})
	}
}
