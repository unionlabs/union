package types

import (
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
)

type Metadata struct {
	Description string `json:"description"`
	// DenomUnits represents the list of DenomUnit's for a given coin
	DenomUnits []DenomUnit `json:"denom_units"`
	// Base represents the base denom (should be the DenomUnit with exponent = 0).
	Base string `json:"base"`
	// Display indicates the suggested denom that should be displayed in clients.
	Display string `json:"display"`
	// Name defines the name of the token (eg: Cosmos Atom)
	Name string `json:"name"`
	// Symbol is the token symbol usually shown on exchanges (eg: ATOM).
	// This can be the same as the display.
	Symbol string `json:"symbol"`
}

type DenomUnit struct {
	// Denom represents the string name of the given denom unit (e.g uatom).
	Denom string `json:"denom"`
	// Exponent represents power of 10 exponent that one must
	// raise the base_denom to in order to equal the given DenomUnit's denom
	// 1 denom = 1^exponent base_denom
	// (e.g. with a base_denom of uatom, one can create a DenomUnit of 'atom' with
	// exponent = 6, thus: 1 atom = 10^6 uatom).
	Exponent uint32 `json:"exponent"`
	// Aliases is a list of string aliases for the given denom
	Aliases []string `json:"aliases"`
}

type Params struct {
	DenomCreationFee []wasmvmtypes.Coin `json:"denom_creation_fee"`
}
