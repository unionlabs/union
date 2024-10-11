package keeper

import (
	"context"

	"cosmossdk.io/core/address"
	v4 "cosmossdk.io/x/slashing/migrations/v4"

	"github.com/cosmos/cosmos-sdk/runtime"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper   Keeper
	valCodec address.ValidatorAddressCodec
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper Keeper, valCodec address.ValidatorAddressCodec) Migrator {
	return Migrator{
		keeper:   keeper,
		valCodec: valCodec,
	}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(_ context.Context) error {
	return nil
}

// Migrate2to3 migrates the x/slashing module state from the consensus
// version 2 to version 3. Specifically, it takes the parameters that are currently stored
// and managed by the x/params modules and stores them directly into the x/slashing
// module state.
func (m Migrator) Migrate2to3(_ context.Context) error {
	return nil
}

// Migrate3to4 migrates the x/slashing module state from the consensus
// version 3 to version 4. Specifically, it migrates the validator missed block
// bitmap.
func (m Migrator) Migrate3to4(ctx context.Context) error {
	store := runtime.KVStoreAdapter(m.keeper.KVStoreService.OpenKVStore(ctx))
	params, err := m.keeper.Params.Get(ctx)
	if err != nil {
		return err
	}
	return v4.Migrate(ctx, m.keeper.cdc, store, params, m.valCodec)
}
