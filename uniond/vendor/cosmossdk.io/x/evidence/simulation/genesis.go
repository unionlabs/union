package simulation

import (
	"encoding/json"
	"fmt"
	"math/rand"

	"cosmossdk.io/x/evidence/exported"
	"cosmossdk.io/x/evidence/types"

	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// Simulation parameter constants
const evidence = "evidence"

// GenEvidences returns an empty slice of evidences.
func GenEvidences(_ *rand.Rand, _ []simtypes.Account) []exported.Evidence {
	return []exported.Evidence{}
}

// RandomizedGenState generates a random GenesisState for evidence
func RandomizedGenState(simState *module.SimulationState) {
	var ev []exported.Evidence

	simState.AppParams.GetOrGenerate(evidence, &ev, simState.Rand, func(r *rand.Rand) { ev = GenEvidences(r, simState.Accounts) })

	evidenceGenesis := types.NewGenesisState(ev)

	bz, err := json.MarshalIndent(&evidenceGenesis, "", " ")
	if err != nil {
		panic(err)
	}
	fmt.Printf("Selected randomly generated %s parameters:\n%s\n", types.ModuleName, bz)
	simState.GenState[types.ModuleName] = simState.Cdc.MustMarshalJSON(evidenceGenesis)
}
