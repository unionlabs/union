package types

import (
	"fmt"
	"strings"

	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"

	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

var (
	// DefaultAllowedClients are the default clients for the AllowedClients parameter.
	DefaultAllowedClients = []string{exported.Solomachine, exported.Tendermint, exported.Wasm, exported.Localhost}

	// KeyAllowedClients is store's key for AllowedClients Params
	KeyAllowedClients = []byte("AllowedClients")
)

// ParamKeyTable type declaration for parameters
func ParamKeyTable() paramtypes.KeyTable {
	return paramtypes.NewKeyTable().RegisterParamSet(&Params{})
}

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

// ParamSetPairs implements params.ParamSet
func (p *Params) ParamSetPairs() paramtypes.ParamSetPairs {
	return paramtypes.ParamSetPairs{
		paramtypes.NewParamSetPair(KeyAllowedClients, p.AllowedClients, validateClients),
	}
}

// IsAllowedClient checks if the given client type is registered on the allowlist.
func (p Params) IsAllowedClient(clientType string) bool {
	for _, allowedClient := range p.AllowedClients {
		if allowedClient == clientType {
			return true
		}
	}
	return false
}

func validateClients(i interface{}) error {
	clients, ok := i.([]string)
	if !ok {
		return fmt.Errorf("invalid parameter type: %T", i)
	}

	for i, clientType := range clients {
		if strings.TrimSpace(clientType) == "" {
			return fmt.Errorf("client type %d cannot be blank", i)
		}
	}

	return nil
}
