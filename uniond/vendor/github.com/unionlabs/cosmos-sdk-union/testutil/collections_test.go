package testutil_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/testutil"
)

func TestDiffCollectionsMigration(t *testing.T) {
	key := storetypes.NewKVStoreKey("test")
	ctx := testutil.DefaultContext(key, storetypes.NewTransientStoreKey("transient"))

	// First try with some invalid hash
	err := testutil.DiffCollectionsMigration(
		ctx,
		key,
		5,
		func(i int64) {
			ctx.KVStore(key).Set([]byte{byte(i)}, []byte{byte(i)})
		},
		"abcdef0123456789",
	)
	require.Error(t, err)

	// Now reset and try with the correct hash
	ctx = testutil.DefaultContext(key, storetypes.NewTransientStoreKey("transient"))
	err = testutil.DiffCollectionsMigration(
		ctx,
		key,
		5,
		func(i int64) {
			ctx.KVStore(key).Set([]byte{byte(i)}, []byte{byte(i)})
		},
		"79541ed9da9c16cb7a1d43d5a3d5f6ee31a873c85a6cb4334fb99e021ee0e556",
	)
	require.NoError(t, err)

	// Change the data a little and it will result in an error
	ctx = testutil.DefaultContext(key, storetypes.NewTransientStoreKey("transient"))
	err = testutil.DiffCollectionsMigration(
		ctx,
		key,
		5,
		func(i int64) {
			ctx.KVStore(key).Set([]byte{byte(i)}, []byte{byte(i + 1)})
		},
		"79541ed9da9c16cb7a1d43d5a3d5f6ee31a873c85a6cb4334fb99e021ee0e556",
	)
	require.Error(t, err)
}
