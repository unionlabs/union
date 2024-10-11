package types

import (
	"errors"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// DefaultDefaultSendEnabled is the value that DefaultSendEnabled will have from DefaultParams().
var DefaultDefaultSendEnabled = true

// NewParams creates a new parameter configuration for the bank module
func NewParams(defaultSendEnabled bool) Params {
	return Params{
		SendEnabled:        nil,
		DefaultSendEnabled: defaultSendEnabled,
	}
}

// DefaultParams is the default parameter configuration for the bank module
func DefaultParams() Params {
	return Params{
		SendEnabled:        nil,
		DefaultSendEnabled: DefaultDefaultSendEnabled,
	}
}

// Validate all bank module parameters
func (p Params) Validate() error {
	if len(p.SendEnabled) > 0 {
		return errors.New("use of send_enabled in params is no longer supported")
	}
	return validateIsBool(p.DefaultSendEnabled)
}

// Validate gets any errors with this SendEnabled entry.
func (se SendEnabled) Validate() error {
	return sdk.ValidateDenom(se.Denom)
}

// NewSendEnabled creates a new SendEnabled object
// The denom may be left empty to control the global default setting of send_enabled
func NewSendEnabled(denom string, sendEnabled bool) *SendEnabled {
	return &SendEnabled{
		Denom:   denom,
		Enabled: sendEnabled,
	}
}

// validateIsBool is used by the x/params module to validate that a thing is a bool.
func validateIsBool(i interface{}) error {
	_, ok := i.(bool)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}
	return nil
}
