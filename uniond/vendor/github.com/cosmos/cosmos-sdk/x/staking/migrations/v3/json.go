package v3

import "github.com/cosmos/cosmos-sdk/x/staking/types"

// MigrateJSON accepts exported v0.43 x/stakinng genesis state and migrates it to
// v0.46 x/staking genesis state. The migration includes:
//
// - Add MinCommissionRate param.
func MigrateJSON(oldState types.GenesisState) (types.GenesisState, error) {
	oldState.Params.MinCommissionRate = types.DefaultMinCommissionRate

	return oldState, nil
}
