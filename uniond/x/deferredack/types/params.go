package types

import (
	"fmt"

	sdkmath "cosmossdk.io/math"
)

// DefaultFeePercentage is the default value used to extract a fee from all forwarded packets.
var DefaultFeePercentage = sdkmath.LegacyNewDec(0)

// NewParams creates a new parameter configuration for the pfm module.
func NewParams(feePercentage sdkmath.LegacyDec) Params {
	return Params{
		FeePercentage: feePercentage,
	}
}

// DefaultParams is the default parameter configuration for the pfm module.
func DefaultParams() Params {
	return NewParams(DefaultFeePercentage)
}

// Validate the pfm module parameters.
func (p Params) Validate() error {
	return validateFeePercentage(p.FeePercentage)
}

// validateFeePercentage asserts that the fee percentage param is a valid sdk.Dec type.
func validateFeePercentage(i interface{}) error {
	v, ok := i.(sdkmath.LegacyDec)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}
	if v.IsNegative() {
		return fmt.Errorf("invalid fee percentage. expected not negative, got %d", v.RoundInt64())
	}
	if !(v.LTE(sdkmath.LegacyOneDec())) {
		return fmt.Errorf("invalid fee percentage. expected less than one 1 got %d", v.RoundInt64())
	}

	return nil
}
