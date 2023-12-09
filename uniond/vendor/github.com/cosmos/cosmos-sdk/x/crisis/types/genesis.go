package types

import (
	"fmt"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NewGenesisState creates a new GenesisState object
func NewGenesisState(constantFee sdk.Coin) *GenesisState {
	return &GenesisState{
		ConstantFee: constantFee,
	}
}

// DefaultGenesisState creates a default GenesisState object
func DefaultGenesisState() *GenesisState {
	return &GenesisState{
		ConstantFee: sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(1000)),
	}
}

// ValidateGenesis - validate crisis genesis data
func ValidateGenesis(data *GenesisState) error {
	if !data.ConstantFee.IsValid() {
		return fmt.Errorf("constant fee is invalid")
	}
	if !data.ConstantFee.IsPositive() {
		return fmt.Errorf("constant fee must be positive: %s", data.ConstantFee)
	}
	return nil
}
