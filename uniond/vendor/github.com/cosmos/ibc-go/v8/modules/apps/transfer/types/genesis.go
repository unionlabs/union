package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

// NewGenesisState creates a new ibc-transfer GenesisState instance.
func NewGenesisState(portID string, denomTraces Traces, params Params, totalEscrowed sdk.Coins) *GenesisState {
	return &GenesisState{
		PortId:        portID,
		DenomTraces:   denomTraces,
		Params:        params,
		TotalEscrowed: totalEscrowed,
	}
}

// DefaultGenesisState returns a GenesisState with "transfer" as the default PortID.
func DefaultGenesisState() *GenesisState {
	return &GenesisState{
		PortId:        PortID,
		DenomTraces:   Traces{},
		Params:        DefaultParams(),
		TotalEscrowed: sdk.Coins{},
	}
}

// Validate performs basic genesis state validation returning an error upon any
// failure.
func (gs GenesisState) Validate() error {
	if err := host.PortIdentifierValidator(gs.PortId); err != nil {
		return err
	}
	if err := gs.DenomTraces.Validate(); err != nil {
		return err
	}
	return gs.TotalEscrowed.Validate() // will fail if there are duplicates for any denom
}
