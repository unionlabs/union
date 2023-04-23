package utils

import (
	"strings"

	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

// NormalizeVoteOption - normalize user specified vote option
func NormalizeVoteOption(option string) string {
	switch option {
	case "Yes", "yes":
		return v1beta1.OptionYes.String()

	case "Abstain", "abstain":
		return v1beta1.OptionAbstain.String()

	case "No", "no":
		return v1beta1.OptionNo.String()

	case "NoWithVeto", "no_with_veto":
		return v1beta1.OptionNoWithVeto.String()

	default:
		return option
	}
}

// NormalizeWeightedVoteOptions - normalize vote options param string
func NormalizeWeightedVoteOptions(options string) string {
	newOptions := []string{}
	for _, option := range strings.Split(options, ",") {
		fields := strings.Split(option, "=")
		fields[0] = NormalizeVoteOption(fields[0])
		if len(fields) < 2 {
			fields = append(fields, "1")
		}
		newOptions = append(newOptions, strings.Join(fields, "="))
	}
	return strings.Join(newOptions, ",")
}

// NormalizeProposalType - normalize user specified proposal type.
func NormalizeProposalType(proposalType string) string {
	switch proposalType {
	case "Text", "text":
		return v1beta1.ProposalTypeText

	default:
		return ""
	}
}

// NormalizeProposalStatus - normalize user specified proposal status.
func NormalizeProposalStatus(status string) string {
	switch status {
	case "DepositPeriod", "deposit_period":
		return v1beta1.StatusDepositPeriod.String()
	case "VotingPeriod", "voting_period":
		return v1beta1.StatusVotingPeriod.String()
	case "Passed", "passed":
		return v1beta1.StatusPassed.String()
	case "Rejected", "rejected":
		return v1beta1.StatusRejected.String()
	default:
		return status
	}
}
