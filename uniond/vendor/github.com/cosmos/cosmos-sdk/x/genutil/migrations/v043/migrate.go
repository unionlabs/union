package v043

import (
	"github.com/cosmos/cosmos-sdk/client"
	v1bank "github.com/cosmos/cosmos-sdk/x/bank/migrations/v1"
	v2bank "github.com/cosmos/cosmos-sdk/x/bank/migrations/v2"
	bank "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
	v1gov "github.com/cosmos/cosmos-sdk/x/gov/migrations/v1"
	v2gov "github.com/cosmos/cosmos-sdk/x/gov/migrations/v2"
	gov "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
)

// Migrate migrates exported state from v0.40 to a v0.43 genesis state.
func Migrate(appState types.AppMap, clientCtx client.Context) (types.AppMap, error) {
	// Migrate x/gov.
	if appState[v1gov.ModuleName] != nil {
		// unmarshal relative source genesis application state
		var oldGovState gov.GenesisState
		clientCtx.Codec.MustUnmarshalJSON(appState[v1gov.ModuleName], &oldGovState)

		// delete deprecated x/gov genesis state
		delete(appState, v1gov.ModuleName)

		// Migrate relative source genesis application state and marshal it into
		// the respective key.
		appState[v2gov.ModuleName] = clientCtx.Codec.MustMarshalJSON(v2gov.MigrateJSON(&oldGovState))
	}

	if appState[v1bank.ModuleName] != nil {
		// unmarshal relative source genesis application state
		var oldBankState bank.GenesisState
		clientCtx.Codec.MustUnmarshalJSON(appState[v1bank.ModuleName], &oldBankState)

		// delete deprecated x/bank genesis state
		delete(appState, v1bank.ModuleName)

		// Migrate relative source genesis application state and marshal it into
		// the respective key.
		appState[v2bank.ModuleName] = clientCtx.Codec.MustMarshalJSON(v2bank.MigrateJSON(&oldBankState))
	}

	return appState, nil
}
