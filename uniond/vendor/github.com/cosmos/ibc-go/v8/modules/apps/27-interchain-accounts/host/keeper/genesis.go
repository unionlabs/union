package keeper

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	genesistypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/genesis/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

// InitGenesis initializes the interchain accounts host application state from a provided genesis state
func InitGenesis(ctx sdk.Context, keeper Keeper, state genesistypes.HostGenesisState) {
	keeper.setPort(ctx, state.Port)

	// generate port capability if it does not already exist
	if !keeper.hasCapability(ctx, state.Port) {
		// use the port keeper to generate a new capability
		capability := keeper.portKeeper.BindPort(ctx, state.Port)

		// use the host scoped keeper to claim the port capability
		if err := keeper.ClaimCapability(ctx, capability, host.PortPath(state.Port)); err != nil {
			panic(fmt.Errorf("could not claim port capability: %v", err))
		}
	}

	for _, ch := range state.ActiveChannels {
		keeper.SetActiveChannelID(ctx, ch.ConnectionId, ch.PortId, ch.ChannelId)
	}

	for _, acc := range state.InterchainAccounts {
		keeper.SetInterchainAccountAddress(ctx, acc.ConnectionId, acc.PortId, acc.AccountAddress)
	}

	if err := state.Params.Validate(); err != nil {
		panic(fmt.Errorf("could not set ica host params at genesis: %v", err))
	}
	keeper.SetParams(ctx, state.Params)
}

// ExportGenesis returns the interchain accounts host exported genesis
func ExportGenesis(ctx sdk.Context, keeper Keeper) genesistypes.HostGenesisState {
	return genesistypes.NewHostGenesisState(
		keeper.GetAllActiveChannels(ctx),
		keeper.GetAllInterchainAccounts(ctx),
		icatypes.HostPortID,
		keeper.GetParams(ctx),
	)
}
