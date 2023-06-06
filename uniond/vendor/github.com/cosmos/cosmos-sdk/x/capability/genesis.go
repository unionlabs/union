package capability

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/capability/keeper"
	"github.com/cosmos/cosmos-sdk/x/capability/types"
)

// InitGenesis initializes the capability module's state from a provided genesis
// state.
func InitGenesis(ctx sdk.Context, k keeper.Keeper, genState types.GenesisState) {
	if err := k.InitializeIndex(ctx, genState.Index); err != nil {
		panic(err)
	}

	// set owners for each index
	for _, genOwner := range genState.Owners {
		k.SetOwners(ctx, genOwner.Index, genOwner.IndexOwners)
	}
	// initialize in-memory capabilities
	k.InitMemStore(ctx)
}

// ExportGenesis returns the capability module's exported genesis.
func ExportGenesis(ctx sdk.Context, k keeper.Keeper) *types.GenesisState {
	index := k.GetLatestIndex(ctx)
	owners := []types.GenesisOwners{}

	for i := uint64(1); i < index; i++ {
		capabilityOwners, ok := k.GetOwners(ctx, i)
		if !ok || len(capabilityOwners.Owners) == 0 {
			continue
		}

		genOwner := types.GenesisOwners{
			Index:       i,
			IndexOwners: capabilityOwners,
		}
		owners = append(owners, genOwner)
	}

	return &types.GenesisState{
		Index:  index,
		Owners: owners,
	}
}
