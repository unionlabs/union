package simulation_test

import (
	"encoding/json"
	"math/rand"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
	sdkmath "cosmossdk.io/math"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/slashing/simulation"
	"github.com/cosmos/cosmos-sdk/x/slashing/testutil"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

// TestRandomizedGenState tests the normal scenario of applying RandomizedGenState.
// Abonormal scenarios are not tested here.
func TestRandomizedGenState(t *testing.T) {
	var interfaceRegistry codectypes.InterfaceRegistry
	depinject.Inject(testutil.AppConfig, &interfaceRegistry)
	cdc := codec.NewProtoCodec(interfaceRegistry)

	s := rand.NewSource(1)
	r := rand.New(s)

	simState := module.SimulationState{
		AppParams:    make(simtypes.AppParams),
		Cdc:          cdc,
		Rand:         r,
		NumBonded:    3,
		Accounts:     simtypes.RandomAccounts(r, 3),
		InitialStake: sdkmath.NewInt(1000),
		GenState:     make(map[string]json.RawMessage),
	}

	simulation.RandomizedGenState(&simState)

	var slashingGenesis types.GenesisState
	simState.Cdc.MustUnmarshalJSON(simState.GenState[types.ModuleName], &slashingGenesis)

	dec1, _ := sdk.NewDecFromStr("0.600000000000000000")
	dec2, _ := sdk.NewDecFromStr("0.022222222222222222")
	dec3, _ := sdk.NewDecFromStr("0.008928571428571429")

	require.Equal(t, dec1, slashingGenesis.Params.MinSignedPerWindow)
	require.Equal(t, dec2, slashingGenesis.Params.SlashFractionDoubleSign)
	require.Equal(t, dec3, slashingGenesis.Params.SlashFractionDowntime)
	require.Equal(t, int64(720), slashingGenesis.Params.SignedBlocksWindow)
	require.Equal(t, time.Duration(34800000000000), slashingGenesis.Params.DowntimeJailDuration)
	require.Len(t, slashingGenesis.MissedBlocks, 0)
	require.Len(t, slashingGenesis.SigningInfos, 0)
}

// TestRandomizedGenState tests abnormal scenarios of applying RandomizedGenState.
func TestRandomizedGenState1(t *testing.T) {
	var interfaceRegistry codectypes.InterfaceRegistry
	depinject.Inject(testutil.AppConfig, &interfaceRegistry)
	cdc := codec.NewProtoCodec(interfaceRegistry)

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
				Cdc:       cdc,
				Rand:      r,
			}, "assignment to entry in nil map"},
	}

	for _, tt := range tests {
		require.Panicsf(t, func() { simulation.RandomizedGenState(&tt.simState) }, tt.panicMsg)
	}
}
