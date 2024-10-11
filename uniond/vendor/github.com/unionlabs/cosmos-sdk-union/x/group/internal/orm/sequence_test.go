package orm

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/group/errors"

	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
)

func TestSequenceUniqueConstraint(t *testing.T) {
	key := storetypes.NewKVStoreKey("test")
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)

	seq := NewSequence(0x1)
	err := seq.InitVal(store, 2)
	require.NoError(t, err)
	err = seq.InitVal(store, 3)
	require.True(t, errors.ErrORMUniqueConstraint.Is(err))
}

func TestSequenceIncrements(t *testing.T) {
	key := storetypes.NewKVStoreKey("test")
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)

	seq := NewSequence(0x1)
	var i uint64
	for i = 1; i < 10; i++ {
		autoID := seq.NextVal(store)
		assert.Equal(t, i, autoID)
		assert.Equal(t, i, seq.CurVal(store))
	}

	seq = NewSequence(0x1)
	assert.Equal(t, uint64(10), seq.PeekNextVal(store))
	assert.Equal(t, uint64(9), seq.CurVal(store))
}
