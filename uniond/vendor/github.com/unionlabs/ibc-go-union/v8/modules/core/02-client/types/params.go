package types

import (
	"fmt"
	"slices"
	"strings"
)

// DefaultAllowedClients are the default clients for the AllowedClients parameter.
// By default it allows all client types.
var DefaultAllowedClients = []string{AllowAllClients}

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
	// Still need to check for blank client type
	if strings.TrimSpace(clientType) == "" {
		return false
	}

	// Check for allow all client wildcard
	// If exist then allow all type of client
	if len(p.AllowedClients) == 1 && p.AllowedClients[0] == AllowAllClients {
		return true
	}

	return slices.Contains(p.AllowedClients, clientType)
}

// validateClients checks that the given clients are not blank and there are no duplicates.
// If AllowAllClients wildcard (*) is used, then there should no other client types in the allow list
func validateClients(clients []string) error {
	if slices.Contains(clients, AllowAllClients) && len(clients) > 1 {
		return fmt.Errorf("allow list must have only one element because the allow all clients wildcard (%s) is present", AllowAllClients)
	}

	foundClients := make(map[string]bool, len(clients))
	for i, clientType := range clients {
		if strings.TrimSpace(clientType) == "" {
			return fmt.Errorf("client type %d cannot be blank", i)
		}
		if foundClients[clientType] {
			return fmt.Errorf("duplicate client type: %s", clientType)
		}
		foundClients[clientType] = true
	}

	return nil
}
