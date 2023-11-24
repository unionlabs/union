package keeper

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	genesistypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/genesis/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

// InitGenesis initializes the interchain accounts controller application state from a provided genesis state
func InitGenesis(ctx sdk.Context, keeper Keeper, state genesistypes.ControllerGenesisState) {
	for _, portID := range state.Ports {
		keeper.setPort(ctx, portID)

		// generate port capability if it does not already exist
		if !keeper.hasCapability(ctx, portID) {
			// use the port keeper to generate a new capability
			capability := keeper.portKeeper.BindPort(ctx, portID)

			// use the controller scoped keeper to claim the port capability
			if err := keeper.ClaimCapability(ctx, capability, host.PortPath(portID)); err != nil {
				panic(fmt.Errorf("could not claim port capability: %v", err))
			}
		}
	}

	for _, ch := range state.ActiveChannels {
		keeper.SetActiveChannelID(ctx, ch.ConnectionId, ch.PortId, ch.ChannelId)

		if ch.IsMiddlewareEnabled {
			keeper.SetMiddlewareEnabled(ctx, ch.PortId, ch.ConnectionId)
		} else {
			keeper.SetMiddlewareDisabled(ctx, ch.PortId, ch.ConnectionId)
		}
	}

	for _, acc := range state.InterchainAccounts {
		keeper.SetInterchainAccountAddress(ctx, acc.ConnectionId, acc.PortId, acc.AccountAddress)
	}

	keeper.SetParams(ctx, state.Params)
}

// ExportGenesis returns the interchain accounts controller exported genesis
func ExportGenesis(ctx sdk.Context, keeper Keeper) genesistypes.ControllerGenesisState {
	return genesistypes.NewControllerGenesisState(
		keeper.GetAllActiveChannels(ctx),
		keeper.GetAllInterchainAccounts(ctx),
		keeper.GetAllPorts(ctx),
		keeper.GetParams(ctx),
	)
}
