package types

import (
	"fmt"
	"strings"
)

// DefaultIndex is the default capability global index
const DefaultIndex uint64 = 1

// DefaultGenesis returns the default Capability genesis state
func DefaultGenesis() *GenesisState {
	return &GenesisState{
		Index:  DefaultIndex,
		Owners: []GenesisOwners{},
	}
}

// Validate performs basic genesis state validation returning an error upon any
// failure.
func (gs GenesisState) Validate() error {
	// NOTE: index must be greater than 0
	if gs.Index == 0 {
		return fmt.Errorf("capability index must be non-zero")
	}

	for _, genOwner := range gs.Owners {
		if len(genOwner.IndexOwners.Owners) == 0 {
			return fmt.Errorf("empty owners in genesis")
		}

		// all exported existing indices must be between [1, gs.Index)
		if genOwner.Index == 0 || genOwner.Index >= gs.Index {
			return fmt.Errorf("owners exist for index %d outside of valid range: %d-%d", genOwner.Index, 1, gs.Index-1)
		}

		for _, owner := range genOwner.IndexOwners.Owners {
			if strings.TrimSpace(owner.Module) == "" {
				return fmt.Errorf("owner's module cannot be blank: %s", owner)
			}

			if strings.TrimSpace(owner.Name) == "" {
				return fmt.Errorf("owner's name cannot be blank: %s", owner)
			}
		}
	}

	return nil
}
