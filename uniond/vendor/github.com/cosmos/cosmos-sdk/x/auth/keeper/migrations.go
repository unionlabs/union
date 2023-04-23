package keeper

import (
	"github.com/cosmos/gogoproto/grpc"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/exported"
	v2 "github.com/cosmos/cosmos-sdk/x/auth/migrations/v2"
	v3 "github.com/cosmos/cosmos-sdk/x/auth/migrations/v3"
	v4 "github.com/cosmos/cosmos-sdk/x/auth/migrations/v4"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper         AccountKeeper
	queryServer    grpc.Server
	legacySubspace exported.Subspace
}

// NewMigrator returns a new Migrator.
func NewMigrator(keeper AccountKeeper, queryServer grpc.Server, ss exported.Subspace) Migrator {
	return Migrator{keeper: keeper, queryServer: queryServer, legacySubspace: ss}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx sdk.Context) error {
	var iterErr error

	m.keeper.IterateAccounts(ctx, func(account types.AccountI) (stop bool) {
		wb, err := v2.MigrateAccount(ctx, account, m.queryServer)
		if err != nil {
			iterErr = err
			return true
		}

		if wb == nil {
			return false
		}

		m.keeper.SetAccount(ctx, wb)
		return false
	})

	return iterErr
}

// Migrate2to3 migrates from consensus version 2 to version 3. Specifically, for each account
// we index the account's ID to their address.
func (m Migrator) Migrate2to3(ctx sdk.Context) error {
	return v3.MigrateStore(ctx, m.keeper.storeKey, m.keeper.cdc)
}

// Migrate3to4 migrates the x/auth module state from the consensus version 3 to
// version 4. Specifically, it takes the parameters that are currently stored
// and managed by the x/params modules and stores them directly into the x/auth
// module state.
func (m Migrator) Migrate3to4(ctx sdk.Context) error {
	return v4.Migrate(ctx, ctx.KVStore(m.keeper.storeKey), m.legacySubspace, m.keeper.cdc)
}

// V45_SetAccount implements V45_SetAccount
// set the account without map to accAddr to accNumber.
//
// NOTE: This is used for testing purposes only.
func (m Migrator) V45_SetAccount(ctx sdk.Context, acc types.AccountI) error {
	addr := acc.GetAddress()
	store := ctx.KVStore(m.keeper.storeKey)

	bz, err := m.keeper.MarshalAccount(acc)
	if err != nil {
		return err
	}

	store.Set(types.AddressStoreKey(addr), bz)
	return nil
}
