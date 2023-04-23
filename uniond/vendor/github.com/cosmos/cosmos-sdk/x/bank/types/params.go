package types

import (
	"errors"
	"fmt"

	"sigs.k8s.io/yaml"

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

// String implements the Stringer interface.
func (p Params) String() string {
	sendEnabled, _ := yaml.Marshal(p.SendEnabled)
	d := " "
	if len(sendEnabled) > 0 && sendEnabled[0] == '-' {
		d = "\n"
	}
	return fmt.Sprintf("default_send_enabled: %t\nsend_enabled:%s%s", p.DefaultSendEnabled, d, sendEnabled)
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

// String implements stringer interface
func (se SendEnabled) String() string {
	return fmt.Sprintf("denom: %s\nenabled: %t\n", se.Denom, se.Enabled)
}

// validateIsBool is used by the x/params module to validate that a thing is a bool.
func validateIsBool(i interface{}) error {
	_, ok := i.(bool)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}
	return nil
}
