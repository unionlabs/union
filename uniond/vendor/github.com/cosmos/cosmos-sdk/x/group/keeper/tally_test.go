package keeper_test

import (
	"context"
	"time"

	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/group"
)

func (s *TestSuite) TestTally() {
	addrs := s.addrs
	addr2 := addrs[1]

	msgSend1 := &banktypes.MsgSend{
		FromAddress: s.groupPolicyAddr.String(),
		ToAddress:   addr2.String(),
		Amount:      sdk.Coins{sdk.NewInt64Coin("test", 100)},
	}
	proposers := []string{addr2.String()}

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
				proposalId := submitProposal(ctx, s, msgs, proposers)
				_, err := s.groupKeeper.WithdrawProposal(ctx, &group.MsgWithdrawProposal{
					ProposalId: proposalId,
					Address:    proposers[0],
				})
				s.Require().NoError(err)

				return proposalId
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
		spec := spec
		s.Run(msg, func() {
			sdkCtx, _ := s.sdkCtx.CacheContext()
			ctx := sdk.WrapSDKContext(sdkCtx)

			pId := spec.setupProposal(ctx)
			req := &group.QueryTallyResultRequest{
				ProposalId: pId,
			}

			res, err := s.groupKeeper.TallyResult(ctx, req)
			if spec.expErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
				s.Require().Equal(res.Tally, spec.expTallyResult)
			}
		})
	}
}
