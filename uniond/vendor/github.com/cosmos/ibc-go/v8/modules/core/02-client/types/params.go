package types

import (
	"fmt"
	"slices"
	"strings"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// DefaultAllowedClients are the default clients for the AllowedClients parameter.
var DefaultAllowedClients = []string{exported.Solomachine, exported.Tendermint, exported.Wasm, exported.Localhost}

// NewParams creates a new parameter configuration for the ibc client module
func NewParams(allowedClients ...string) Params {
	return Params{
		AllowedClients: allowedClients,
	}
}

// DefaultParams is the default parameter configuration for the ibc-client module.
func DefaultParams() Params {
	return NewParams(DefaultAllowedClients...)
}

// Validate all ibc-client module parameters
func (p Params) Validate() error {
	return validateClients(p.AllowedClients)
}

// IsAllowedClient checks if the given client type is registered on the allowlist.
func (p Params) IsAllowedClient(clientType string) bool {
	return slices.Contains(p.AllowedClients, clientType)
}

// validateClients checks that the given clients are not blank.
func validateClients(clients []string) error {
	for i, clientType := range clients {
		if strings.TrimSpace(clientType) == "" {
			return fmt.Errorf("client type %d cannot be blank", i)
		}
	}

	return nil
}
