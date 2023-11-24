package keeper

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

// InitGenesis initializes the ibc-transfer state and binds to PortID.
func (k Keeper) InitGenesis(ctx sdk.Context, state types.GenesisState) {
	k.SetPort(ctx, state.PortId)

	for _, trace := range state.DenomTraces {
		k.SetDenomTrace(ctx, trace)
		k.setDenomMetadata(ctx, trace)
	}

	// Only try to bind to port if it is not already bound, since we may already own
	// port capability from capability InitGenesis
	if !k.hasCapability(ctx, state.PortId) {
		// transfer module binds to the transfer port on InitChain
		// and claims the returned capability
		err := k.BindPort(ctx, state.PortId)
		if err != nil {
			panic(fmt.Errorf("could not claim port capability: %v", err))
		}
	}

	k.SetParams(ctx, state.Params)

	// Every denom will have only one total escrow amount, since any
	// duplicate entry will fail validation in Validate of GenesisState
	for _, denomEscrow := range state.TotalEscrowed {
		k.SetTotalEscrowForDenom(ctx, denomEscrow)
	}
}

// ExportGenesis exports ibc-transfer module's portID and denom trace info into its genesis state.
func (k Keeper) ExportGenesis(ctx sdk.Context) *types.GenesisState {
	return &types.GenesisState{
		PortId:        k.GetPort(ctx),
		DenomTraces:   k.GetAllDenomTraces(ctx),
		Params:        k.GetParams(ctx),
		TotalEscrowed: k.GetAllTotalEscrowed(ctx),
	}
}
