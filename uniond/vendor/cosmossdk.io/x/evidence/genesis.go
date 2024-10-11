package evidence

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/x/evidence/exported"
	"cosmossdk.io/x/evidence/keeper"
	"cosmossdk.io/x/evidence/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
)

// InitGenesis initializes the evidence module's state from a provided genesis
// state.
func InitGenesis(ctx context.Context, k keeper.Keeper, gs *types.GenesisState) error {
	if err := gs.Validate(); err != nil {
		return fmt.Errorf("failed to validate %s genesis state: %w", types.ModuleName, err)
	}

	for _, e := range gs.Evidence {
		evi, ok := e.GetCachedValue().(exported.Evidence)
		if !ok {
			return errors.New("expected evidence")
		}
		if _, err := k.Evidences.Get(ctx, evi.Hash()); err == nil {
			return fmt.Errorf("evidence with hash %s already exists", evi.Hash())
		}

		if err := k.Evidences.Set(ctx, evi.Hash(), evi); err != nil {
			return err
		}
	}
	return nil
}

// ExportGenesis returns the evidence module's exported genesis.
func ExportGenesis(ctx context.Context, k keeper.Keeper) (*types.GenesisState, error) {
	gs := new(types.GenesisState)
	err := k.Evidences.Walk(ctx, nil, func(_ []byte, value exported.Evidence) (stop bool, err error) {
		anyEvi, err := codectypes.NewAnyWithValue(value)
		if err != nil {
			return false, err
		}
		gs.Evidence = append(gs.Evidence, anyEvi)
		return false, nil
	})
	if err != nil {
		return nil, err
	}
	return gs, nil
}
