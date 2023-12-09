package v047

import (
	"github.com/cosmos/cosmos-sdk/client"
	v1auth "github.com/cosmos/cosmos-sdk/x/auth/migrations/v1"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	bankv4 "github.com/cosmos/cosmos-sdk/x/bank/migrations/v4"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	v1distr "github.com/cosmos/cosmos-sdk/x/distribution/migrations/v1"
	v3distr "github.com/cosmos/cosmos-sdk/x/distribution/migrations/v3"
	distrtypes "github.com/cosmos/cosmos-sdk/x/distribution/types"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
	v4gov "github.com/cosmos/cosmos-sdk/x/gov/migrations/v4"
	govv1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	groupv2 "github.com/cosmos/cosmos-sdk/x/group/migrations/v2"
)

// Migrate migrates exported state from v0.46 to a v0.47 genesis state.
func Migrate(appState types.AppMap, clientCtx client.Context) (types.AppMap, error) {
	// Migrate x/bank.
	bankState := appState[banktypes.ModuleName]
	if len(bankState) > 0 {
		var oldBankState banktypes.GenesisState
		clientCtx.Codec.MustUnmarshalJSON(bankState, &oldBankState)
		newBankState := bankv4.MigrateGenState(&oldBankState)
		appState[banktypes.ModuleName] = clientCtx.Codec.MustMarshalJSON(newBankState)
	}

	if govOldState, ok := appState[v4gov.ModuleName]; ok {
		// unmarshal relative source genesis application state
		var old govv1.GenesisState
		clientCtx.Codec.MustUnmarshalJSON(govOldState, &old)

		// delete deprecated x/gov genesis state
		delete(appState, v4gov.ModuleName)

		// set the x/gov genesis state with new state.
		new, err := v4gov.MigrateJSON(&old)
		if err != nil {
			return nil, err
		}
		appState[v4gov.ModuleName] = clientCtx.Codec.MustMarshalJSON(new)
	}

	// Migrate x/auth group policy accounts
	if authOldState, ok := appState[v1auth.ModuleName]; ok {
		var old authtypes.GenesisState
		clientCtx.Codec.MustUnmarshalJSON(authOldState, &old)
		newAuthState := groupv2.MigrateGenState(&old)
		appState[v1auth.ModuleName] = clientCtx.Codec.MustMarshalJSON(newAuthState)
	}

	// Migrate x/distribution params (reset unused)
	if oldDistState, ok := appState[v1distr.ModuleName]; ok {
		var old distrtypes.GenesisState
		clientCtx.Codec.MustUnmarshalJSON(oldDistState, &old)
		newDistState := v3distr.MigrateJSON(&old)
		appState[v1distr.ModuleName] = clientCtx.Codec.MustMarshalJSON(newDistState)
	}

	return appState, nil
}
