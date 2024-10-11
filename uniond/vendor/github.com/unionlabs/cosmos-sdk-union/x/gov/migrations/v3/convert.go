package v3

import (
	"fmt"

	"cosmossdk.io/math"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

const (
	// ModuleName is the name of the module
	ModuleName = "gov"
)

// ConvertToLegacyProposal takes a new proposal and attempts to convert it to the
// legacy proposal format. This conversion is best effort. New proposal types that
// don't have a legacy message will return a "nil" content.
// Returns error when the amount of messages in `proposal` is different than one.
func ConvertToLegacyProposal(proposal v1.Proposal) (v1beta1.Proposal, error) {
	var err error
	legacyProposal := v1beta1.Proposal{
		ProposalId:   proposal.Id,
		Status:       v1beta1.ProposalStatus(proposal.Status),
		TotalDeposit: types.NewCoins(proposal.TotalDeposit...),
	}

	legacyProposal.FinalTallyResult, err = ConvertToLegacyTallyResult(proposal.FinalTallyResult)
	if err != nil {
		return v1beta1.Proposal{}, err
	}

	if proposal.VotingStartTime != nil {
		legacyProposal.VotingStartTime = *proposal.VotingStartTime
	}

	if proposal.VotingEndTime != nil {
		legacyProposal.VotingEndTime = *proposal.VotingEndTime
	}

	if proposal.SubmitTime != nil {
		legacyProposal.SubmitTime = *proposal.SubmitTime
	}

	if proposal.DepositEndTime != nil {
		legacyProposal.DepositEndTime = *proposal.DepositEndTime
	}

	msgs, err := proposal.GetMsgs()
	if err != nil {
		return v1beta1.Proposal{}, err
	}
	if len(msgs) != 1 {
		return v1beta1.Proposal{}, sdkerrors.ErrInvalidType.Wrap("can't convert a gov/v1 Proposal to gov/v1beta1 Proposal when amount of proposal messages not exactly one")
	}
	if legacyMsg, ok := msgs[0].(*v1.MsgExecLegacyContent); ok {
		// check that the content struct can be unmarshalled
		_, err := v1.LegacyContentFromMessage(legacyMsg)
		if err != nil {
			return v1beta1.Proposal{}, err
		}
		legacyProposal.Content = legacyMsg.Content
		return legacyProposal, nil
	}
	// hack to fill up the content with the first message
	// this is to support clients that have not yet (properly) use gov/v1 endpoints
	// https://github.com/cosmos/cosmos-sdk/issues/14334
	// VerifyBasic assures that we have at least one message.
	legacyProposal.Content, err = codectypes.NewAnyWithValue(msgs[0])

	return legacyProposal, err
}

func ConvertToLegacyTallyResult(tally *v1.TallyResult) (v1beta1.TallyResult, error) {
	yes, ok := math.NewIntFromString(tally.YesCount)
	if !ok {
		return v1beta1.TallyResult{}, fmt.Errorf("unable to convert yes tally string (%s) to int", tally.YesCount)
	}
	no, ok := math.NewIntFromString(tally.NoCount)
	if !ok {
		return v1beta1.TallyResult{}, fmt.Errorf("unable to convert no tally string (%s) to int", tally.NoCount)
	}
	veto, ok := math.NewIntFromString(tally.NoWithVetoCount)
	if !ok {
		return v1beta1.TallyResult{}, fmt.Errorf("unable to convert no with veto tally string (%s) to int", tally.NoWithVetoCount)
	}
	abstain, ok := math.NewIntFromString(tally.AbstainCount)
	if !ok {
		return v1beta1.TallyResult{}, fmt.Errorf("unable to convert abstain tally string (%s) to int", tally.AbstainCount)
	}

	return v1beta1.TallyResult{
		Yes:        yes,
		No:         no,
		NoWithVeto: veto,
		Abstain:    abstain,
	}, nil
}

func ConvertToLegacyVote(vote v1.Vote) (v1beta1.Vote, error) {
	options, err := ConvertToLegacyVoteOptions(vote.Options)
	if err != nil {
		return v1beta1.Vote{}, err
	}
	return v1beta1.Vote{
		ProposalId: vote.ProposalId,
		Voter:      vote.Voter,
		Options:    options,
	}, nil
}

func ConvertToLegacyVoteOptions(voteOptions []*v1.WeightedVoteOption) ([]v1beta1.WeightedVoteOption, error) {
	options := make([]v1beta1.WeightedVoteOption, len(voteOptions))
	for i, option := range voteOptions {
		weight, err := math.LegacyNewDecFromStr(option.Weight)
		if err != nil {
			return options, err
		}
		options[i] = v1beta1.WeightedVoteOption{
			Option: v1beta1.VoteOption(option.Option),
			Weight: weight,
		}
	}
	return options, nil
}

func ConvertToLegacyDeposit(deposit *v1.Deposit) v1beta1.Deposit {
	return v1beta1.Deposit{
		ProposalId: deposit.ProposalId,
		Depositor:  deposit.Depositor,
		Amount:     types.NewCoins(deposit.Amount...),
	}
}
