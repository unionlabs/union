package v3_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	v3 "cosmossdk.io/x/gov/migrations/v3"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
)

func TestConvertToLegacyProposal(t *testing.T) {
	propTime := time.Unix(1e9, 0)
	legacyContentMsg, err := v1.NewLegacyContent(v1beta1.NewTextProposal("title", "description"), "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh")
	require.NoError(t, err)
	msgs := []sdk.Msg{legacyContentMsg}
	msgsAny, err := tx.SetMsgs(msgs)
	require.NoError(t, err)
	proposal := v1.Proposal{
		Id:              1,
		Status:          v1.StatusDepositPeriod,
		Messages:        msgsAny,
		SubmitTime:      &propTime,
		DepositEndTime:  &propTime,
		VotingStartTime: &propTime,
		VotingEndTime:   &propTime,
		Metadata:        "proposal metadata",
	}

	testCases := map[string]struct {
		tallyResult v1.TallyResult
		expErr      bool
	}{
		"valid": {
			tallyResult: v1.EmptyTallyResult(),
		},
		"invalid final tally result": {
			tallyResult: v1.TallyResult{},
			expErr:      true,
		},
	}

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			proposal.FinalTallyResult = &tc.tallyResult
			v1beta1Proposal, err := v3.ConvertToLegacyProposal(proposal)
			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				require.Equal(t, v1beta1Proposal.ProposalId, proposal.Id)
				require.Equal(t, v1beta1Proposal.VotingStartTime, *proposal.VotingStartTime)
				require.Equal(t, v1beta1Proposal.VotingEndTime, *proposal.VotingEndTime)
				require.Equal(t, v1beta1Proposal.SubmitTime, *proposal.SubmitTime)
				require.Equal(t, v1beta1Proposal.DepositEndTime, *proposal.DepositEndTime)
				require.Equal(t, v1beta1Proposal.FinalTallyResult.Yes, math.NewInt(0))
				require.Equal(t, v1beta1Proposal.FinalTallyResult.No, math.NewInt(0))
				require.Equal(t, v1beta1Proposal.FinalTallyResult.NoWithVeto, math.NewInt(0))
				require.Equal(t, v1beta1Proposal.FinalTallyResult.Abstain, math.NewInt(0))
				tp, ok := v1beta1Proposal.Content.GetCachedValue().(*v1beta1.TextProposal)
				require.Truef(t, ok, "expected *TextProposal, got %T", v1beta1Proposal.Content.GetCachedValue())
				require.Equal(t, tp.Title, "title")
				require.Equal(t, tp.Description, "description")
			}
		})
	}
}

func TestConvertToLegacyProposalContent(t *testing.T) {
	msg := v1.MsgCancelProposal{ProposalId: 1, Proposer: "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh"}
	msgsAny, err := tx.SetMsgs([]sdk.Msg{&msg})
	require.NoError(t, err)
	tallyResult := v1.EmptyTallyResult()
	proposal := v1.Proposal{
		Id:               1,
		Status:           v1.StatusDepositPeriod,
		Messages:         msgsAny,
		Metadata:         "proposal metadata",
		FinalTallyResult: &tallyResult,
	}

	legacyP, err := v3.ConvertToLegacyProposal(proposal)
	require.NoError(t, err)
	tp, ok := legacyP.Content.GetCachedValue().(*v1.MsgCancelProposal)
	require.Truef(t, ok, "expected *MsgCancelProposal, got %T", legacyP.Content.GetCachedValue())
	require.Equal(t, &msg, tp)

	// more than one message is not supported
	proposal.Messages, err = tx.SetMsgs([]sdk.Msg{&msg, &msg})
	require.NoError(t, err)
	_, err = v3.ConvertToLegacyProposal(proposal)
	require.ErrorIs(t, sdkerrors.ErrInvalidType, err)

	// zero messages is not supported
	proposal.Messages = nil
	_, err = v3.ConvertToLegacyProposal(proposal)
	require.ErrorIs(t, sdkerrors.ErrInvalidType, err)
}

func TestConvertToLegacyTallyResult(t *testing.T) {
	tallyResult := v1.EmptyTallyResult()
	testCases := map[string]struct {
		tallyResult v1.TallyResult
		expErr      bool
	}{
		"valid": {
			tallyResult: tallyResult,
		},
		"invalid yes count": {
			tallyResult: v1.TallyResult{
				YesCount:        "invalid",
				NoCount:         tallyResult.NoCount,
				AbstainCount:    tallyResult.AbstainCount,
				NoWithVetoCount: tallyResult.NoWithVetoCount,
				SpamCount:       tallyResult.SpamCount,
			},
			expErr: true,
		},
		"invalid no count": {
			tallyResult: v1.TallyResult{
				YesCount:        tallyResult.YesCount,
				NoCount:         "invalid",
				AbstainCount:    tallyResult.AbstainCount,
				NoWithVetoCount: tallyResult.NoWithVetoCount,
				SpamCount:       tallyResult.SpamCount,
			},
			expErr: true,
		},
		"invalid abstain count": {
			tallyResult: v1.TallyResult{
				YesCount:        tallyResult.YesCount,
				NoCount:         tallyResult.NoCount,
				AbstainCount:    "invalid",
				NoWithVetoCount: tallyResult.NoWithVetoCount,
				SpamCount:       tallyResult.SpamCount,
			},
			expErr: true,
		},
		"invalid no with veto count": {
			tallyResult: v1.TallyResult{
				YesCount:        tallyResult.YesCount,
				NoCount:         tallyResult.NoCount,
				AbstainCount:    tallyResult.AbstainCount,
				NoWithVetoCount: "invalid",
				SpamCount:       tallyResult.SpamCount,
			},
			expErr: true,
		},
	}
	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			_, err := v3.ConvertToLegacyTallyResult(&tc.tallyResult)
			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestConvertToLegacyVote(t *testing.T) {
	vote := v1.Vote{
		ProposalId: 1,
		Voter:      "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh",
		Metadata:   "vote metadata",
	}

	testCases := map[string]struct {
		options []*v1.WeightedVoteOption
		expErr  bool
	}{
		"valid": {
			options: v1.NewNonSplitVoteOption(v1.OptionYes),
		},
		"invalid options": {
			options: []*v1.WeightedVoteOption{{Option: 1, Weight: "invalid"}},
			expErr:  true,
		},
	}

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			vote.Options = tc.options
			v1beta1Vote, err := v3.ConvertToLegacyVote(vote)
			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				require.Equal(t, v1beta1Vote.ProposalId, vote.ProposalId)
				require.Equal(t, v1beta1Vote.Voter, vote.Voter)
				require.Equal(t, v1beta1Vote.Options[0].Option, v1beta1.OptionYes)
				require.Equal(t, v1beta1Vote.Options[0].Weight, math.LegacyNewDec(1))
			}
		})
	}
}

func TestConvertToLegacyDeposit(t *testing.T) {
	deposit := v1.Deposit{
		ProposalId: 1,
		Depositor:  "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh",
		Amount:     sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(1))),
	}

	v1beta1Deposit := v3.ConvertToLegacyDeposit(&deposit)
	require.Equal(t, v1beta1Deposit.ProposalId, deposit.ProposalId)
	require.Equal(t, v1beta1Deposit.Depositor, deposit.Depositor)
	require.Equal(t, v1beta1Deposit.Amount[0], deposit.Amount[0])
}
