package types

import (
	errorsmod "cosmossdk.io/errors"
)

// NewGenesisState creates an 08-wasm GenesisState instance.
func NewGenesisState(contracts []Contract) *GenesisState {
	return &GenesisState{Contracts: contracts}
}

// Validate performs basic genesis state validation returning an error upon any
// failure.
func (gs GenesisState) Validate() error {
	for _, contract := range gs.Contracts {
		if err := ValidateWasmCode(contract.CodeBytes); err != nil {
			return errorsmod.Wrap(err, "wasm bytecode validation failed")
		}
	}

	return nil
}
