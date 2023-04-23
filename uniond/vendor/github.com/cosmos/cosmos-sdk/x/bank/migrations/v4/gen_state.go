package v4

import (
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

// MigrateGenState accepts exported v0.43 x/bank genesis state and migrates it to
// v0.47 x/bank genesis state. The migration includes:
// - Move the SendEnabled entries from Params to the new GenesisState.SendEnabled field.
func MigrateGenState(oldState *types.GenesisState) *types.GenesisState {
	newState := *oldState
	newState.MigrateSendEnabled()
	return &newState
}
