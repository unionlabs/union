package utils

import (
	"strings"

	v1 "cosmossdk.io/x/gov/types/v1"
)

// NormalizeVoteOption - normalize user specified vote option
func NormalizeVoteOption(option string) string {
	switch {
	case strings.EqualFold(option, "yes"):
		return v1.OptionYes.String()

	case strings.EqualFold(option, "abstain"):
		return v1.OptionAbstain.String()

	case strings.EqualFold(option, "no"):
		return v1.OptionNo.String()

	case strings.EqualFold(option, "nowithveto"):
		return v1.OptionNoWithVeto.String()

	case strings.EqualFold(option, "spam"):
		return v1.OptionSpam.String()
	}

	// extra check for no_with_veto
	if option == "no_with_veto" || option == "no-with-veto" {
		return v1.OptionNoWithVeto.String()
	}

	return option
}

// NormalizeProposalType - normalize user specified proposal type.
func NormalizeProposalType(proposalType string) v1.ProposalType {
	switch proposalType {
	case "Expedited", "expedited":
		return v1.ProposalType_PROPOSAL_TYPE_EXPEDITED
	case "MultipleChoice", "multiple_choice", "multiple-choice":
		return v1.ProposalType_PROPOSAL_TYPE_MULTIPLE_CHOICE
	case "Optimistic", "optimistic":
		return v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC
	default:
		return v1.ProposalType_PROPOSAL_TYPE_STANDARD
	}
}

// NormalizeWeightedVoteOptions - normalize vote options param string
func NormalizeWeightedVoteOptions(options string) string {
	tmpOptions := strings.Split(options, ",")
	newOptions := make([]string, 0, len(tmpOptions))
	for _, option := range tmpOptions {
		fields := strings.Split(option, "=")
		fields[0] = NormalizeVoteOption(fields[0])
		if len(fields) < 2 {
			fields = append(fields, "1")
		}
		newOptions = append(newOptions, strings.Join(fields, "="))
	}
	return strings.Join(newOptions, ",")
}

// NormalizeProposalStatus - normalize user specified proposal status.
func NormalizeProposalStatus(status string) string {
	switch status {
	case "DepositPeriod", "deposit_period":
		return v1.StatusDepositPeriod.String()
	case "VotingPeriod", "voting_period":
		return v1.StatusVotingPeriod.String()
	case "Passed", "passed":
		return v1.StatusPassed.String()
	case "Rejected", "rejected":
		return v1.StatusRejected.String()
	default:
		return status
	}
}
