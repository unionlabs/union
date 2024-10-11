package testutil

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"cosmossdk.io/core/header"
	corestore "cosmossdk.io/core/store"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	"cosmossdk.io/store"
	"cosmossdk.io/store/metrics"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// DefaultContext creates a sdk.Context with a fresh MemDB that can be used in tests.
func DefaultContext(key, tkey storetypes.StoreKey) sdk.Context {
	db := coretesting.NewMemDB()
	cms := store.NewCommitMultiStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	cms.MountStoreWithDB(key, storetypes.StoreTypeIAVL, db)
	cms.MountStoreWithDB(tkey, storetypes.StoreTypeTransient, db)
	err := cms.LoadLatestVersion()
	if err != nil {
		panic(err)
	}
	ctx := sdk.NewContext(cms, false, log.NewNopLogger())

	return ctx
}

// DefaultContextWithKeys creates a sdk.Context with a fresh MemDB, mounting the providing keys for usage in the multistore.
// This function is intended to be used for testing purposes only.
func DefaultContextWithKeys(
	keys map[string]*storetypes.KVStoreKey,
	transKeys map[string]*storetypes.TransientStoreKey,
	memKeys map[string]*storetypes.MemoryStoreKey,
) sdk.Context {
	db := coretesting.NewMemDB()
	cms := store.NewCommitMultiStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())

	for _, key := range keys {
		cms.MountStoreWithDB(key, storetypes.StoreTypeIAVL, db)
	}

	for _, tKey := range transKeys {
		cms.MountStoreWithDB(tKey, storetypes.StoreTypeTransient, db)
	}

	for _, memkey := range memKeys {
		cms.MountStoreWithDB(memkey, storetypes.StoreTypeMemory, db)
	}

	err := cms.LoadLatestVersion()
	if err != nil {
		panic(err)
	}

	return sdk.NewContext(cms, false, log.NewNopLogger())
}

type TestContext struct {
	Ctx sdk.Context
	DB  corestore.KVStoreWithBatch
	CMS store.CommitMultiStore
}

func DefaultContextWithDB(tb testing.TB, key, tkey storetypes.StoreKey) TestContext {
	tb.Helper()
	db := coretesting.NewMemDB()
	cms := store.NewCommitMultiStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	cms.MountStoreWithDB(key, storetypes.StoreTypeIAVL, db)
	cms.MountStoreWithDB(tkey, storetypes.StoreTypeTransient, db)
	err := cms.LoadLatestVersion()
	assert.NoError(tb, err)

	ctx := sdk.NewContext(cms, false, log.NewNopLogger()).WithHeaderInfo(header.Info{Time: time.Now()})

	return TestContext{ctx, db, cms}
}
