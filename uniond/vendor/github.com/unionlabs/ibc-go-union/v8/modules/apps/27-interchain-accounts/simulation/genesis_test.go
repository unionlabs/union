package simulation_test

import (
	"encoding/json"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	genesistypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/genesis/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/simulation"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
)

// TestRandomizedGenState tests the normal scenario of applying RandomizedGenState.
// Abonormal scenarios are not tested here.
func TestRandomizedGenState(t *testing.T) {
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	cryptocodec.RegisterInterfaces(interfaceRegistry)
	cdc := codec.NewProtoCodec(interfaceRegistry)

	s := rand.NewSource(1) // 1 is the seed
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

	var icaGenesis genesistypes.GenesisState
	simState.Cdc.MustUnmarshalJSON(simState.GenState[types.ModuleName], &icaGenesis)

	require.True(t, icaGenesis.ControllerGenesisState.Params.ControllerEnabled)
	require.Empty(t, icaGenesis.ControllerGenesisState.ActiveChannels)
	require.Empty(t, icaGenesis.ControllerGenesisState.InterchainAccounts)
	require.Empty(t, icaGenesis.ControllerGenesisState.Ports)

	require.True(t, icaGenesis.HostGenesisState.Params.HostEnabled)
	require.Equal(t, []string{"*"}, icaGenesis.HostGenesisState.Params.AllowMessages)
	require.Equal(t, types.HostPortID, icaGenesis.HostGenesisState.Port)
	require.Empty(t, icaGenesis.ControllerGenesisState.ActiveChannels)
	require.Empty(t, icaGenesis.ControllerGenesisState.InterchainAccounts)
}
