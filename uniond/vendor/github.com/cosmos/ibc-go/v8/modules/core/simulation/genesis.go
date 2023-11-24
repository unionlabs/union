package simulation

// DONTCOVER

import (
	"encoding/json"
	"fmt"
	"math/rand"

	"github.com/cosmos/cosmos-sdk/types/module"

	clientsims "github.com/cosmos/ibc-go/v8/modules/core/02-client/simulation"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectionsims "github.com/cosmos/ibc-go/v8/modules/core/03-connection/simulation"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channelsims "github.com/cosmos/ibc-go/v8/modules/core/04-channel/simulation"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	"github.com/cosmos/ibc-go/v8/modules/core/types"
)

// Simulation parameter constants
const (
	clientGenesis     = "client_genesis"
	connectionGenesis = "connection_genesis"
	channelGenesis    = "channel_genesis"
)

// RandomizedGenState generates a random GenesisState for evidence
func RandomizedGenState(simState *module.SimulationState) {
	var (
		clientGenesisState     clienttypes.GenesisState
		connectionGenesisState connectiontypes.GenesisState
		channelGenesisState    channeltypes.GenesisState
	)

	simState.AppParams.GetOrGenerate(
		clientGenesis, &clientGenesisState, simState.Rand,
		func(r *rand.Rand) { clientGenesisState = clientsims.GenClientGenesis(r, simState.Accounts) },
	)

	simState.AppParams.GetOrGenerate(
		connectionGenesis, &connectionGenesisState, simState.Rand,
		func(r *rand.Rand) { connectionGenesisState = connectionsims.GenConnectionGenesis(r, simState.Accounts) },
	)

	simState.AppParams.GetOrGenerate(
		channelGenesis, &channelGenesisState, simState.Rand,
		func(r *rand.Rand) { channelGenesisState = channelsims.GenChannelGenesis(r, simState.Accounts) },
	)

	ibcGenesis := types.GenesisState{
		ClientGenesis:     clientGenesisState,
		ConnectionGenesis: connectionGenesisState,
		ChannelGenesis:    channelGenesisState,
	}

	bz, err := json.MarshalIndent(&ibcGenesis, "", " ")
	if err != nil {
		panic(err)
	}
	fmt.Printf("Selected randomly generated %s parameters:\n%s\n", ibcexported.ModuleName, bz)
	simState.GenState[ibcexported.ModuleName] = simState.Cdc.MustMarshalJSON(&ibcGenesis)
}
