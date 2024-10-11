package simulation

import (
	"math/rand"

	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
)

// GenConnectionGenesis returns the default connection genesis state.
func GenConnectionGenesis(_ *rand.Rand, _ []simtypes.Account) types.GenesisState {
	return types.DefaultGenesisState()
}
