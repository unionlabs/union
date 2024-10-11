package v1

import (
	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// ValidatorGovInfo used for tallying
type ValidatorGovInfo struct {
	Address             sdk.ValAddress      // address of the validator operator
	BondedTokens        math.Int            // Power of a Validator
	DelegatorShares     math.LegacyDec      // Total outstanding delegator shares
	DelegatorDeductions math.LegacyDec      // Delegator deductions from validator's delegators voting independently
	Vote                WeightedVoteOptions // Vote of the validator
}

// NewValidatorGovInfo creates a ValidatorGovInfo instance
func NewValidatorGovInfo(address sdk.ValAddress, bondedTokens math.Int, delegatorShares,
	delegatorDeductions math.LegacyDec, options WeightedVoteOptions,
) ValidatorGovInfo {
	return ValidatorGovInfo{
		Address:             address,
		BondedTokens:        bondedTokens,
		DelegatorShares:     delegatorShares,
		DelegatorDeductions: delegatorDeductions,
		Vote:                options,
	}
}

// NewTallyResult creates a new TallyResult instance
func NewTallyResult(option1, option2, option3, option4, spam math.Int) TallyResult {
	return TallyResult{
		YesCount:         option1.String(), // deprecated, kept for client backwards compatibility
		AbstainCount:     option2.String(), // deprecated, kept for client backwards compatibility
		NoCount:          option3.String(), // deprecated, kept for client backwards compatibility
		NoWithVetoCount:  option4.String(), // deprecated, kept for client backwards compatibility
		OptionOneCount:   option1.String(),
		OptionTwoCount:   option2.String(),
		OptionThreeCount: option3.String(),
		OptionFourCount:  option4.String(),
		SpamCount:        spam.String(),
	}
}

// NewTallyResultFromMap creates a new TallyResult instance from a Option -> Dec map
func NewTallyResultFromMap(results map[VoteOption]math.LegacyDec) TallyResult {
	return NewTallyResult(
		results[OptionYes].TruncateInt(),
		results[OptionAbstain].TruncateInt(),
		results[OptionNo].TruncateInt(),
		results[OptionNoWithVeto].TruncateInt(),
		results[OptionSpam].TruncateInt(),
	)
}

// EmptyTallyResult returns an empty TallyResult.
func EmptyTallyResult() TallyResult {
	return NewTallyResult(math.ZeroInt(), math.ZeroInt(), math.ZeroInt(), math.ZeroInt(), math.ZeroInt())
}

// Equals returns if two tally results are equal.
func (tr TallyResult) Equals(comp TallyResult) bool {
	return tr.YesCount == comp.YesCount &&
		tr.AbstainCount == comp.AbstainCount &&
		tr.NoCount == comp.NoCount &&
		tr.NoWithVetoCount == comp.NoWithVetoCount &&
		tr.OptionOneCount == comp.OptionOneCount &&
		tr.OptionTwoCount == comp.OptionTwoCount &&
		tr.OptionThreeCount == comp.OptionThreeCount &&
		tr.OptionFourCount == comp.OptionFourCount &&
		tr.SpamCount == comp.SpamCount
}
