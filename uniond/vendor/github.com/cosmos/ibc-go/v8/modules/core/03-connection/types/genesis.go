package types

import (
	"fmt"

	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

// NewConnectionPaths creates a ConnectionPaths instance.
func NewConnectionPaths(id string, paths []string) ConnectionPaths {
	return ConnectionPaths{
		ClientId: id,
		Paths:    paths,
	}
}

// NewGenesisState creates a GenesisState instance.
func NewGenesisState(
	connections []IdentifiedConnection, connPaths []ConnectionPaths,
	nextConnectionSequence uint64, params Params,
) GenesisState {
	return GenesisState{
		Connections:            connections,
		ClientConnectionPaths:  connPaths,
		NextConnectionSequence: nextConnectionSequence,
		Params:                 params,
	}
}

// DefaultGenesisState returns the ibc connection submodule's default genesis state.
func DefaultGenesisState() GenesisState {
	return GenesisState{
		Connections:            []IdentifiedConnection{},
		ClientConnectionPaths:  []ConnectionPaths{},
		NextConnectionSequence: 0,
		Params:                 DefaultParams(),
	}
}

// Validate performs basic genesis state validation returning an error upon any
// failure.
func (gs GenesisState) Validate() error {
	// keep track of the max sequence to ensure it is less than
	// the next sequence used in creating connection identifers.
	var maxSequence uint64

	for i, conn := range gs.Connections {
		sequence, err := ParseConnectionSequence(conn.Id)
		if err != nil {
			return err
		}

		if sequence > maxSequence {
			maxSequence = sequence
		}

		if err := conn.ValidateBasic(); err != nil {
			return fmt.Errorf("invalid connection %v index %d: %w", conn, i, err)
		}
	}

	for i, conPaths := range gs.ClientConnectionPaths {
		if err := host.ClientIdentifierValidator(conPaths.ClientId); err != nil {
			return fmt.Errorf("invalid client connection path %d: %w", i, err)
		}
		for _, connectionID := range conPaths.Paths {
			if err := host.ConnectionIdentifierValidator(connectionID); err != nil {
				return fmt.Errorf("invalid client connection ID (%s) in connection paths %d: %w", connectionID, i, err)
			}
		}
	}

	if maxSequence != 0 && maxSequence >= gs.NextConnectionSequence {
		return fmt.Errorf("next connection sequence %d must be greater than maximum sequence used in connection identifier %d", gs.NextConnectionSequence, maxSequence)
	}

	return gs.Params.Validate()
}
