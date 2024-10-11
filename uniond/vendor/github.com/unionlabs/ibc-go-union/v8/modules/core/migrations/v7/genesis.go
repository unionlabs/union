package v7

import (
	"github.com/cosmos/cosmos-sdk/codec"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"

	clientv7 "github.com/cosmos/ibc-go/v8/modules/core/02-client/migrations/v7"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	"github.com/cosmos/ibc-go/v8/modules/core/types"
)

// MigrateGenesis accepts an exported IBC client genesis file and migrates it to:
//
// - Update solo machine client state protobuf definition (v2 to v3)
// - Remove all solo machine consensus states
// - Remove any localhost clients
func MigrateGenesis(appState genutiltypes.AppMap, cdc codec.ProtoCodecMarshaler) (genutiltypes.AppMap, error) {
	if appState[ibcexported.ModuleName] == nil {
		return appState, nil
	}

	// ensure legacy solo machines types are registered
	clientv7.RegisterInterfaces(cdc.InterfaceRegistry())

	// unmarshal old ibc genesis state
	ibcGenState := &types.GenesisState{}
	cdc.MustUnmarshalJSON(appState[ibcexported.ModuleName], ibcGenState)

	clientGenState, err := clientv7.MigrateGenesis(&ibcGenState.ClientGenesis, cdc)
	if err != nil {
		return nil, err
	}

	ibcGenState.ClientGenesis = *clientGenState

	// delete old genesis state
	delete(appState, ibcexported.ModuleName)

	// set new ibc genesis state
	appState[ibcexported.ModuleName] = cdc.MustMarshalJSON(ibcGenState)
	return appState, nil
}
