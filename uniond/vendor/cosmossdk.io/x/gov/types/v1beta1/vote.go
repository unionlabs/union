package v1beta1

import (
	"fmt"
	"strings"

	sdkmath "cosmossdk.io/math"
)

// Empty returns whether a vote is empty.
func (v Vote) Empty() bool {
	return v.String() == (&Vote{}).String()
}

// Votes is an array of vote
type Votes []Vote

// Equal returns true if two slices (order-dependant) of votes are equal.
func (v Votes) Equal(other Votes) bool {
	if len(v) != len(other) {
		return false
	}

	for i, vote := range v {
		if vote.String() != other[i].String() {
			return false
		}
	}

	return true
}

// String implements stringer interface
func (v Votes) String() string {
	if len(v) == 0 {
		return "[]"
	}
	out := fmt.Sprintf("Votes for Proposal %d:", v[0].ProposalId)
	for _, vot := range v {
		out += fmt.Sprintf("\n  %s: %s", vot.Voter, vot.Options)
	}
	return out
}

// NewNonSplitVoteOption creates a single option vote with weight 1
func NewNonSplitVoteOption(option VoteOption) WeightedVoteOptions {
	return WeightedVoteOptions{{option, sdkmath.LegacyNewDec(1)}}
}

// WeightedVoteOptions describes array of WeightedVoteOptions
type WeightedVoteOptions []WeightedVoteOption

func (v WeightedVoteOptions) String() (out string) {
	for _, opt := range v {
		out += opt.String() + "\n"
	}

	return strings.TrimSpace(out)
}

// ValidWeightedVoteOption returns true if the sub vote is valid and false otherwise.
func ValidWeightedVoteOption(option WeightedVoteOption) bool {
	if !option.Weight.IsPositive() || option.Weight.GT(sdkmath.LegacyNewDec(1)) {
		return false
	}
	return ValidVoteOption(option.Option)
}

// VoteOptionFromString returns a VoteOption from a string. It returns an error
// if the string is invalid.
func VoteOptionFromString(str string) (VoteOption, error) {
	option, ok := VoteOption_value[str]
	if !ok {
		return OptionEmpty, fmt.Errorf("'%s' is not a valid vote option, available options: yes/no/no_with_veto/abstain", str)
	}
	return VoteOption(option), nil
}

// WeightedVoteOptionsFromString returns weighted vote options from string. It returns an error
// if the string is invalid.
func WeightedVoteOptionsFromString(str string) (WeightedVoteOptions, error) {
	options := WeightedVoteOptions{}
	for _, option := range strings.Split(str, ",") {
		fields := strings.Split(option, "=")
		option, err := VoteOptionFromString(fields[0])
		if err != nil {
			return options, err
		}
		if len(fields) < 2 {
			return options, fmt.Errorf("weight field does not exist for %s option", fields[0])
		}
		weight, err := sdkmath.LegacyNewDecFromStr(fields[1])
		if err != nil {
			return options, err
		}
		options = append(options, WeightedVoteOption{option, weight})
	}
	return options, nil
}

// ValidVoteOption returns true if the vote option is valid and false otherwise.
func ValidVoteOption(option VoteOption) bool {
	if option == OptionYes ||
		option == OptionAbstain ||
		option == OptionNo ||
		option == OptionNoWithVeto {
		return true
	}
	return false
}

// Format implements the fmt.Formatter interface.
func (vo VoteOption) Format(s fmt.State, verb rune) {
	switch verb {
	case 's':
		_, _ = s.Write([]byte(vo.String()))
	default:
		_, _ = s.Write([]byte(fmt.Sprintf("%v", byte(vo))))
	}
}
