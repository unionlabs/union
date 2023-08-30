package types

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func NewParams(denomCreationFee sdk.Coins) Params {
	return Params{
		DenomCreationFee: denomCreationFee,
	}
}

// default tokenfactory module parameters.
func DefaultParams() Params {
	return Params{
		DenomCreationFee:        sdk.NewCoins(sdk.NewInt64Coin(sdk.DefaultBondDenom, 10_000_000)),
		DenomCreationGasConsume: 2_000_000,
	}
}

// validate params.
func (p Params) Validate() error {
	err := validateDenomCreationFee(p.DenomCreationFee)

	return err
}

func validateDenomCreationFee(i interface{}) error {
	v, ok := i.(sdk.Coins)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	if v.Validate() != nil {
		return fmt.Errorf("invalid denom creation fee: %+v", i)
	}

	return nil
}

func validateDenomCreationFeeGasConsume(i interface{}) error {
	_, ok := i.(uint64)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	return nil
}
