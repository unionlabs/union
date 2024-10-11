package keeper

import (
	"context"

	v4 "cosmossdk.io/x/distribution/migrations/v4"
	"cosmossdk.io/x/distribution/types"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper Keeper
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper Keeper) Migrator {
	return Migrator{keeper: keeper}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx context.Context) error {
	return nil
}

// Migrate2to3 migrates the x/distribution module state from the consensus
// version 2 to version 3. Specifically, it takes the parameters that are currently stored
// and managed by the x/params module and stores them directly into the x/distribution
// module state.
func (m Migrator) Migrate2to3(ctx context.Context) error {
	return nil
}

// Migrate3to4 migrates the x/distribution module state to use collections
// Additionally it migrates distribution fee pool to use protocol pool module account
func (m Migrator) Migrate3to4(ctx context.Context) error {
	if err := v4.MigrateStore(ctx, m.keeper.Environment, m.keeper.cdc); err != nil {
		return err
	}

	return m.migrateFunds(ctx)
}

func (m Migrator) migrateFunds(ctx context.Context) error {
	macc := m.keeper.GetDistributionAccount(ctx)
	poolMacc := m.keeper.authKeeper.GetModuleAccount(ctx, types.ProtocolPoolDistrAccount)

	feePool, err := m.keeper.FeePool.Get(ctx)
	if err != nil {
		return err
	}

	feePool, err = v4.MigrateFunds(ctx, m.keeper.bankKeeper, feePool, macc, poolMacc)
	if err != nil {
		return err
	}

	// the feePool has now an empty community pool and the remainder is stored in the DecimalPool
	return m.keeper.FeePool.Set(ctx, feePool)
}
