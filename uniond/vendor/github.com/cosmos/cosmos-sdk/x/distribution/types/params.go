package types

import (
	"fmt"

	"cosmossdk.io/math"
)

// DefaultParams returns default distribution parameters
func DefaultParams() Params {
	return Params{
		CommunityTax:        math.LegacyNewDecWithPrec(2, 2), // 2%
		BaseProposerReward:  math.LegacyZeroDec(),            // deprecated
		BonusProposerReward: math.LegacyZeroDec(),            // deprecated
		WithdrawAddrEnabled: true,
	}
}

// ValidateBasic performs basic validation on distribution parameters.
func (p Params) ValidateBasic() error {
	return validateCommunityTax(p.CommunityTax)
}

func validateCommunityTax(i interface{}) error {
	v, ok := i.(math.LegacyDec)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v.IsNil() {
		return fmt.Errorf("community tax must be not nil")
	}
	if v.IsNegative() {
		return fmt.Errorf("community tax must be positive: %s", v)
	}
	if v.GT(math.LegacyOneDec()) {
		return fmt.Errorf("community tax too large: %s", v)
	}

	return nil
}

func validateWithdrawAddrEnabled(i interface{}) error {
	_, ok := i.(bool)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	return nil
}
