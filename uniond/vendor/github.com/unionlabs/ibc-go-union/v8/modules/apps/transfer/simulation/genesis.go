package simulation

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"strings"

	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

// Simulation parameter constants
const port = "port_id"

// RadomEnabled randomized send or receive enabled param with 75% prob of being true.
func RadomEnabled(r *rand.Rand) bool {
	return r.Int63n(101) <= 75
}

// RandomizedGenState generates a random GenesisState for transfer.
func RandomizedGenState(simState *module.SimulationState) {
	var portID string
	simState.AppParams.GetOrGenerate(
		port, &portID, simState.Rand,
		func(r *rand.Rand) { portID = strings.ToLower(simtypes.RandStringOfLength(r, 20)) },
	)

	var sendEnabled bool
	simState.AppParams.GetOrGenerate(
		string(types.KeySendEnabled), &sendEnabled, simState.Rand,
		func(r *rand.Rand) { sendEnabled = RadomEnabled(r) },
	)

	var receiveEnabled bool
	simState.AppParams.GetOrGenerate(
		string(types.KeyReceiveEnabled), &receiveEnabled, simState.Rand,
		func(r *rand.Rand) { receiveEnabled = RadomEnabled(r) },
	)

	transferGenesis := types.GenesisState{
		PortId:      portID,
		DenomTraces: types.Traces{},
		Params:      types.NewParams(sendEnabled, receiveEnabled),
	}

	bz, err := json.MarshalIndent(&transferGenesis, "", " ")
	if err != nil {
		panic(err)
	}
	fmt.Printf("Selected randomly generated %s parameters:\n%s\n", types.ModuleName, bz)
	simState.GenState[types.ModuleName] = simState.Cdc.MustMarshalJSON(&transferGenesis)
}
