package simulation_test

import (
	"encoding/json"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	sdkmath "cosmossdk.io/math"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/mint"
	"github.com/cosmos/cosmos-sdk/x/mint/simulation"
	"github.com/cosmos/cosmos-sdk/x/mint/types"
)

// TestRandomizedGenState tests the normal scenario of applying RandomizedGenState.
// Abonormal scenarios are not tested here.
func TestRandomizedGenState(t *testing.T) {
	encCfg := moduletestutil.MakeTestEncodingConfig(mint.AppModuleBasic{})

	s := rand.NewSource(1)
	r := rand.New(s)

	simState := module.SimulationState{
		AppParams:    make(simtypes.AppParams),
		Cdc:          encCfg.Codec,
		Rand:         r,
		NumBonded:    3,
		Accounts:     simtypes.RandomAccounts(r, 3),
		InitialStake: sdkmath.NewInt(1000),
		GenState:     make(map[string]json.RawMessage),
	}

	simulation.RandomizedGenState(&simState)

	var mintGenesis types.GenesisState
	simState.Cdc.MustUnmarshalJSON(simState.GenState[types.ModuleName], &mintGenesis)

	dec1, _ := sdk.NewDecFromStr("0.670000000000000000")
	dec2, _ := sdk.NewDecFromStr("0.200000000000000000")
	dec3, _ := sdk.NewDecFromStr("0.070000000000000000")

	require.Equal(t, uint64(6311520), mintGenesis.Params.BlocksPerYear)
	require.Equal(t, dec1, mintGenesis.Params.GoalBonded)
	require.Equal(t, dec2, mintGenesis.Params.InflationMax)
	require.Equal(t, dec3, mintGenesis.Params.InflationMin)
	require.Equal(t, "stake", mintGenesis.Params.MintDenom)
	require.Equal(t, "0stake", mintGenesis.Minter.BlockProvision(mintGenesis.Params).String())
	require.Equal(t, "0.170000000000000000", mintGenesis.Minter.NextAnnualProvisions(mintGenesis.Params, math.OneInt()).String())
	require.Equal(t, "0.169999926644441493", mintGenesis.Minter.NextInflationRate(mintGenesis.Params, math.LegacyOneDec()).String())
	require.Equal(t, "0.170000000000000000", mintGenesis.Minter.Inflation.String())
	require.Equal(t, "0.000000000000000000", mintGenesis.Minter.AnnualProvisions.String())
}

// TestRandomizedGenState tests abnormal scenarios of applying RandomizedGenState.
func TestRandomizedGenState1(t *testing.T) {
	encCfg := moduletestutil.MakeTestEncodingConfig(mint.AppModuleBasic{})

	s := rand.NewSource(1)
	r := rand.New(s)
	// all these tests will panic
	tests := []struct {
		simState module.SimulationState
		panicMsg string
	}{
		{ // panic => reason: incomplete initialization of the simState
			module.SimulationState{}, "invalid memory address or nil pointer dereference"},
		{ // panic => reason: incomplete initialization of the simState
			module.SimulationState{
				AppParams: make(simtypes.AppParams),
				Cdc:       encCfg.Codec,
				Rand:      r,
			}, "assignment to entry in nil map"},
	}

	for _, tt := range tests {
		require.Panicsf(t, func() { simulation.RandomizedGenState(&tt.simState) }, tt.panicMsg)
	}
}
