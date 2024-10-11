package types

import (
	"context"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/math"
)

// InflationCalculationFn defines the function required to calculate inflation rate during
// BeginBlock. It receives the minter and params stored in the keeper, along with the current
// bondedRatio and returns the newly calculated inflation rate.
// It can be used to specify a custom inflation calculation logic, instead of relying on the
// default logic provided by the sdk.
// Deprecated: use MintFn instead.
type InflationCalculationFn func(ctx context.Context, minter Minter, params Params, bondedRatio math.LegacyDec) math.LegacyDec

// MintFn defines the function that needs to be implemented in order to customize the minting process.
type MintFn func(ctx context.Context, env appmodule.Environment, minter *Minter, epochId string, epochNumber int64) error

// DefaultInflationCalculationFn is the default function used to calculate inflation.
// Deprecated: use DefaultMintFn instead.
func DefaultInflationCalculationFn(_ context.Context, minter Minter, params Params, bondedRatio math.LegacyDec) math.LegacyDec {
	return minter.NextInflationRate(params, bondedRatio)
}

// NewGenesisState creates a new GenesisState object
func NewGenesisState(minter Minter, params Params) *GenesisState {
	return &GenesisState{
		Minter: minter,
		Params: params,
	}
}

// DefaultGenesisState creates a default GenesisState object
func DefaultGenesisState() *GenesisState {
	return &GenesisState{
		Minter: DefaultInitialMinter(),
		Params: DefaultParams(),
	}
}

// ValidateGenesis validates the provided genesis state to ensure the
// expected invariants holds.
func ValidateGenesis(data GenesisState) error {
	if err := data.Params.Validate(); err != nil {
		return err
	}

	return ValidateMinter(data.Minter)
}
