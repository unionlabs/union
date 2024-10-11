package simulation

import (
	"encoding/json"
	"fmt"
	"math/rand"

	"github.com/cosmos/cosmos-sdk/types/module"

	controllertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	genesistypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/genesis/types"
	hosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
)

// RandomEnabled randomized controller or host enabled param with 75% prob of being true.
func RandomEnabled(r *rand.Rand) bool {
	return r.Int63n(101) <= 75
}

// RandomizedGenState generates a random GenesisState for ics27.
// Only the params are non nil
func RandomizedGenState(simState *module.SimulationState) {
	var controllerEnabled bool
	simState.AppParams.GetOrGenerate(
		string(controllertypes.KeyControllerEnabled), &controllerEnabled, simState.Rand,
		func(r *rand.Rand) { controllerEnabled = RandomEnabled(r) },
	)

	controllerParams := controllertypes.Params{
		ControllerEnabled: controllerEnabled,
	}

	controllerGenesisState := genesistypes.ControllerGenesisState{
		ActiveChannels:     nil,
		InterchainAccounts: nil,
		Ports:              []string{},
		Params:             controllerParams,
	}

	var hostEnabled bool
	simState.AppParams.GetOrGenerate(
		string(hosttypes.KeyHostEnabled), &hostEnabled, simState.Rand,
		func(r *rand.Rand) { hostEnabled = RandomEnabled(r) },
	)

	hostParams := hosttypes.Params{
		HostEnabled:   hostEnabled,
		AllowMessages: []string{"*"}, // allow all messages
	}

	hostGenesisState := genesistypes.HostGenesisState{
		ActiveChannels:     nil,
		InterchainAccounts: nil,
		Port:               types.HostPortID,
		Params:             hostParams,
	}

	icaGenesis := genesistypes.GenesisState{
		ControllerGenesisState: controllerGenesisState,
		HostGenesisState:       hostGenesisState,
	}

	bz, err := json.MarshalIndent(&icaGenesis, "", " ")
	if err != nil {
		panic(err)
	}
	fmt.Printf("Selected randomly generated %s parameters:\n%s\n", types.ModuleName, bz)
	simState.GenState[types.ModuleName] = simState.Cdc.MustMarshalJSON(&icaGenesis)
}
