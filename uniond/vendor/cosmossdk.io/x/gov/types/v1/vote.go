package v1

import (
	"encoding/json"
	"fmt"
	"strings"

	"cosmossdk.io/math"
)

const (
	OptionEmpty = VoteOption_VOTE_OPTION_UNSPECIFIED
	OptionOne   = VoteOption_VOTE_OPTION_ONE
	OptionTwo   = VoteOption_VOTE_OPTION_TWO
	OptionThree = VoteOption_VOTE_OPTION_THREE
	OptionFour  = VoteOption_VOTE_OPTION_FOUR
	OptionSpam  = VoteOption_VOTE_OPTION_SPAM

	OptionYes        = VoteOption_VOTE_OPTION_YES
	OptionNo         = VoteOption_VOTE_OPTION_NO
	OptionNoWithVeto = VoteOption_VOTE_OPTION_NO_WITH_VETO
	OptionAbstain    = VoteOption_VOTE_OPTION_ABSTAIN
)

// NewVote creates a new Vote instance
func NewVote(proposalID uint64, voter string, options WeightedVoteOptions, metadata string) Vote {
	return Vote{ProposalId: proposalID, Voter: voter, Options: options, Metadata: metadata}
}

// Empty returns whether a vote is empty.
func (v Vote) Empty() bool {
	return v.ProposalId == 0 || v.Voter == "" || len(v.Options) == 0
}

// Votes is a collection of Vote objects
type Votes []*Vote

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

func NewWeightedVoteOption(option VoteOption, weight math.LegacyDec) *WeightedVoteOption {
	return &WeightedVoteOption{Option: option, Weight: weight.String()}
}

// IsValid returns true if the sub vote is valid and false otherwise.
func (w *WeightedVoteOption) IsValid() bool {
	weight, err := math.LegacyNewDecFromStr(w.Weight)
	if err != nil {
		return false
	}

	if !weight.IsPositive() || weight.GT(math.LegacyNewDec(1)) {
		return false
	}

	return ValidVoteOption(w.Option)
}

// NewNonSplitVoteOption creates a single option vote with weight 1
func NewNonSplitVoteOption(option VoteOption) WeightedVoteOptions {
	return WeightedVoteOptions{{option, math.LegacyNewDec(1).String()}}
}

// ValidWeightedVoteOption returns true if the sub vote is valid and false otherwise.
func ValidWeightedVoteOption(option WeightedVoteOption) bool {
	weight, err := math.LegacyNewDecFromStr(option.Weight)
	if err != nil || !weight.IsPositive() || weight.GT(math.LegacyNewDec(1)) {
		return false
	}
	return ValidVoteOption(option.Option)
}

// WeightedVoteOptions describes array of WeightedVoteOptions
type WeightedVoteOptions []*WeightedVoteOption

func (v WeightedVoteOptions) String() string {
	out, _ := json.Marshal(v)
	return string(out)
}

// VoteOptionFromString returns a VoteOption from a string. It returns an error
// if the string is invalid.
func VoteOptionFromString(str string) (VoteOption, error) {
	option, ok := VoteOption_value[str]
	if !ok {
		return OptionEmpty, fmt.Errorf("'%s' is not a valid vote option, available options: yes,option_one/no,option_three/no_with_veto,option_four/abstain,option_two/spam", str)
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
		weight, err := math.LegacyNewDecFromStr(fields[1])
		if err != nil {
			return options, err
		}
		options = append(options, NewWeightedVoteOption(option, weight))
	}
	return options, nil
}

// ValidVoteOption returns true if the vote option is valid and false otherwise.
func ValidVoteOption(option VoteOption) bool {
	if option == OptionOne ||
		option == OptionTwo ||
		option == OptionThree ||
		option == OptionFour ||
		option == OptionSpam {
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
