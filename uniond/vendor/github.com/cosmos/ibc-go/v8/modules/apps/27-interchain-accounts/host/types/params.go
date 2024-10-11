package types

import (
	"fmt"
	"strings"
)

const (
	// DefaultHostEnabled is the default value for the host param (set to true)
	DefaultHostEnabled = true
)

// NewParams creates a new parameter configuration for the host submodule
func NewParams(enableHost bool, allowMsgs []string) Params {
	return Params{
		HostEnabled:   enableHost,
		AllowMessages: allowMsgs,
	}
}

// DefaultParams is the default parameter configuration for the host submodule
func DefaultParams() Params {
	return NewParams(DefaultHostEnabled, []string{AllowAllHostMsgs})
}

// Validate validates all host submodule parameters
func (p Params) Validate() error {
	return validateAllowlist(p.AllowMessages)
}

func validateAllowlist(allowMsgs []string) error {
	for _, typeURL := range allowMsgs {
		if strings.TrimSpace(typeURL) == "" {
			return fmt.Errorf("parameter must not contain empty strings: %s", allowMsgs)
		}
	}

	return nil
}
