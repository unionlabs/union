package types

import (
	"fmt"

	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"
)

const (
	// DefaultControllerEnabled is the default value for the controller param (set to true)
	DefaultControllerEnabled = true
)

// KeyControllerEnabled is the store key for ControllerEnabled Params
var KeyControllerEnabled = []byte("ControllerEnabled")

// ParamKeyTable type declaration for parameters
func ParamKeyTable() paramtypes.KeyTable {
	return paramtypes.NewKeyTable().RegisterParamSet(&Params{})
}

// NewParams creates a new parameter configuration for the controller submodule
func NewParams(enableController bool) Params {
	return Params{
		ControllerEnabled: enableController,
	}
}

// DefaultParams is the default parameter configuration for the controller submodule
func DefaultParams() Params {
	return NewParams(DefaultControllerEnabled)
}

// Validate validates all controller submodule parameters
func (p Params) Validate() error {
	if err := validateEnabledType(p.ControllerEnabled); err != nil {
		return err
	}

	return nil
}

// ParamSetPairs implements params.ParamSet
func (p *Params) ParamSetPairs() paramtypes.ParamSetPairs {
	return paramtypes.ParamSetPairs{
		paramtypes.NewParamSetPair(KeyControllerEnabled, p.ControllerEnabled, validateEnabledType),
	}
}

func validateEnabledType(i interface{}) error {
	_, ok := i.(bool)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	return nil
}
