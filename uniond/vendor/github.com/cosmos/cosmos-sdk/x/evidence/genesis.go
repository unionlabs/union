package evidence

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/evidence/exported"
	"github.com/cosmos/cosmos-sdk/x/evidence/keeper"
	"github.com/cosmos/cosmos-sdk/x/evidence/types"
)

// InitGenesis initializes the evidence module's state from a provided genesis
// state.
func InitGenesis(ctx sdk.Context, k keeper.Keeper, gs *types.GenesisState) {
	if err := gs.Validate(); err != nil {
		panic(fmt.Sprintf("failed to validate %s genesis state: %s", types.ModuleName, err))
	}

	for _, e := range gs.Evidence {
		evi, ok := e.GetCachedValue().(exported.Evidence)
		if !ok {
			panic("expected evidence")
		}
		if _, ok := k.GetEvidence(ctx, evi.Hash()); ok {
			panic(fmt.Sprintf("evidence with hash %s already exists", evi.Hash()))
		}

		k.SetEvidence(ctx, evi)
	}
}

// ExportGenesis returns the evidence module's exported genesis.
func ExportGenesis(ctx sdk.Context, k keeper.Keeper) *types.GenesisState {
	e := k.GetAllEvidence(ctx)
	evidence := make([]*codectypes.Any, len(e))
	for i, evi := range e {
		msg, ok := evi.(proto.Message)
		if !ok {
			panic(fmt.Errorf("cannot proto marshal %T", evi))
		}
		any, err := codectypes.NewAnyWithValue(msg)
		if err != nil {
			panic(err)
		}
		evidence[i] = any
	}
	return &types.GenesisState{
		Evidence: evidence,
	}
}
