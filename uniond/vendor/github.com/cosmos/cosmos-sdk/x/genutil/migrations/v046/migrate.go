package v046

import (
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
	v2gov "github.com/cosmos/cosmos-sdk/x/gov/migrations/v2"
	v3gov "github.com/cosmos/cosmos-sdk/x/gov/migrations/v3"
	govv1beta1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	stakingv2 "github.com/cosmos/cosmos-sdk/x/staking/migrations/v2"
	stakingv3 "github.com/cosmos/cosmos-sdk/x/staking/migrations/v3"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

// Migrate migrates exported state from v0.43 to a v0.46 genesis state.
func Migrate(appState types.AppMap, clientCtx client.Context) (types.AppMap, error) {
	// Migrate x/gov.
	if appState[v2gov.ModuleName] != nil {
		// unmarshal relative source genesis application state
		var old govv1beta1.GenesisState
		clientCtx.Codec.MustUnmarshalJSON(appState[v2gov.ModuleName], &old)

		// delete deprecated x/gov genesis state
		delete(appState, v2gov.ModuleName)

		// Migrate relative source genesis application state and marshal it into
		// the respective key.
		new, err := v3gov.MigrateJSON(&old)
		if err != nil {
			return nil, err
		}
		appState[v3gov.ModuleName] = clientCtx.Codec.MustMarshalJSON(new)
	}

	// Migrate x/staking.
	if appState[stakingv2.ModuleName] != nil {
		// unmarshal relative source genesis application state
		var old stakingtypes.GenesisState
		clientCtx.Codec.MustUnmarshalJSON(appState[stakingv2.ModuleName], &old)

		// delete deprecated x/staking genesis state
		delete(appState, stakingv2.ModuleName)

		// Migrate relative source genesis application state and marshal it into
		// the respective key.
		new, err := stakingv3.MigrateJSON(old)
		if err != nil {
			return nil, err
		}
		appState[stakingv3.ModuleName] = clientCtx.Codec.MustMarshalJSON(&new)
	}

	return appState, nil
}
